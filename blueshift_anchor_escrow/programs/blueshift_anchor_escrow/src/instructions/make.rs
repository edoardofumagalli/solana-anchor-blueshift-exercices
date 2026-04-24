use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked},
};

use crate::{errors::EscrowError, state::Escrow};

// `Make` defines every account this instruction needs.
// Anchor validates these constraints before `handler` runs.
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Make<'info> {
    // The maker creates the escrow and pays rent for new accounts.
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        init,
        payer = maker,
        // Allocate enough space for the account discriminator plus Escrow data.
        space = Escrow::INIT_SPACE + Escrow::DISCRIMINATOR.len(),
        // Derive a PDA so each maker can create distinct escrows with different seeds.
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump,
    )]
    pub escrow: Account<'info, Escrow>,

    // Mint of the token the maker is depositing into escrow.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    // Mint of the token the maker expects to receive from the taker later.
    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    // The maker must already own an ATA for mint A because tokens are moved out of it.
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    // This ATA is the escrow "vault".
    // Its authority is the escrow PDA, so the deposited tokens are held by the program.
    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    // Programs required by the account constraints and CPI transfer below.
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
}

impl<'info> Make<'info> {
    // Save the escrow terms on-chain so future instructions can enforce them.
    fn populate_escrow(&mut self, seed: u64, receive_amount: u64, bump: u8) -> Result<()> {
        self.escrow.set_inner(Escrow {
            seed,
            maker: self.maker.key(),
            mint_a: self.mint_a.key(),
            mint_b: self.mint_b.key(),
            // `receive` is the amount of mint B the maker wants in exchange.
            receive: receive_amount,
            bump,
        });

        Ok(())
    }

    // Move the maker's deposit of mint A into the vault using a CPI to the token program.
    fn deposit_tokens(&self, amount: u64) -> Result<()> {
        transfer_checked(
            CpiContext::new(
                self.token_program.to_account_info(),
                TransferChecked {
                    from: self.maker_ata_a.to_account_info(),
                    mint: self.mint_a.to_account_info(),
                    to: self.vault.to_account_info(),
                    authority: self.maker.to_account_info(),
                },
            ),
            amount,
            self.mint_a.decimals,
        )?;

        Ok(())
    }
}

// `receive` is how much token B the maker wants back.
// `amount` is how much token A the maker deposits into the vault now.
pub fn handler(ctx: Context<Make>, seed: u64, receive_amount: u64, amount: u64) -> Result<()> {
    // A zero-value escrow would be meaningless, so reject it early.
    require_gt!(receive_amount, 0, EscrowError::InvalidAmount);
    require_gt!(amount, 0, EscrowError::InvalidAmount);

    // Write the escrow configuration into the PDA account.
    ctx.accounts
        .populate_escrow(seed, receive_amount, ctx.bumps.escrow)?;

    // After the state is created, transfer the maker's tokens into the program-controlled vault.
    ctx.accounts.deposit_tokens(amount)?;

    Ok(())
}
