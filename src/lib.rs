use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWxqSW5FzL2u9U14mZnEKTVvDZp9"); // 실제 배포할 때는 변경 필요

#[program]
pub mod solana_lock_contract {
    use super::*;

    const UNLOCK_TIMESTAMP: i64 = 1821465600; // 2027년 12월 24일 (유닉스 타임스탬프)

    #[derive(Accounts)]
    pub struct Deposit<'info> {
        #[account(mut)]
        pub contract_account: Signer<'info>, // 컨트랙트가 SOL을 보관
        #[account(mut)]
        pub user: Signer<'info>, // 입금하는 사용자
        pub system_program: Program<'info, System>,
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.user.key(),
            &ctx.accounts.contract_account.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.user.to_account_info(),
                ctx.accounts.contract_account.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        Ok(())
    }

    #[derive(Accounts)]
    pub struct Withdraw<'info> {
        #[account(mut, address = contract_account.key())]
        pub contract_account: Signer<'info>,
        #[account(mut, address = contract_account.key())] // 배포자 주소로만 출금 가능
        pub withdraw_to: Signer<'info>,
        pub system_program: Program<'info, System>,
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let current_timestamp = Clock::get()?.unix_timestamp;
        require!(current_timestamp >= UNLOCK_TIMESTAMP, CustomError::WithdrawNotAllowedYet);

        let contract_balance = ctx.accounts.contract_account.to_account_info().lamports();
        require!(contract_balance > 0, CustomError::NoFundsAvailable);

        **ctx.accounts.contract_account.to_account_info().lamports.borrow_mut() -= contract_balance;
        **ctx.accounts.withdraw_to.to_account_info().lamports.borrow_mut() += contract_balance;

        Ok(())
    }

    #[error_code]
    pub enum CustomError {
        #[msg("Withdrawal is not allowed yet.")]
        WithdrawNotAllowedYet,
        #[msg("No funds available to withdraw.")]
        NoFundsAvailable,
    }
}
