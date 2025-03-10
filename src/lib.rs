use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    clock::Clock,
    sysvar::Sysvar,
    rent::Rent,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct ContractState {
    pub owner: Pubkey,
    pub amount: u64,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let account = next_account_info(accounts_iter)?;
    let payer = next_account_info(accounts_iter)?;

    let mut contract_state = ContractState::try_from_slice(&account.data.borrow())?;
    let rent = Rent::get()?;
    let rent_exempt_amount = rent.minimum_balance(account.data_len());

    if contract_state.owner == Pubkey::default() {
        contract_state.owner = *payer.key;
    }

    match instruction_data[0] {
        0 => {
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
            **account.lamports.borrow_mut() += amount;
            **payer.lamports.borrow_mut() -= amount;
            contract_state.amount += amount;
            msg!("Deposited {} lamports", amount);
        }
        1 => {
            if payer.key != &contract_state.owner {
                return Err(ProgramError::MissingRequiredSignature);
            }
            let current_time = Clock::get()?.unix_timestamp;
            let lock_time = 1766966400; // 2027-12-24
            if current_time < lock_time {
                return Err(ProgramError::InvalidAccountData);
            }
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().map_err(|_| ProgramError::InvalidInstructionData)?);
            if amount > contract_state.amount || account.lamports() - amount < rent_exempt_amount {
                return Err(ProgramError::InsufficientFunds);
            }
            **account.lamports.borrow_mut() -= amount;
            **payer.lamports.borrow_mut() += amount;
            contract_state.amount -= amount;
            msg!("Withdrew {} lamports", amount);
        }
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    contract_state.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}

impl ContractState {
    fn try_from_slice(data: &[u8]) -> Result<Self, ProgramError> {
        if data.len() < 40 { return Err(ProgramError::InvalidAccountData); }
        Ok(ContractState {
            owner: Pubkey::new_from_array(data[0..32].try_into().unwrap()),
            amount: u64::from_le_bytes(data[32..40].try_into().unwrap()),
        })
    }

    fn serialize(&self, output: &mut [u8]) -> Result<(), ProgramError> {
        if output.len() < 40 { return Err(ProgramError::InvalidAccountData); }
        output[0..32].copy_from_slice(&self.owner.to_bytes());
        output[32..40].copy_from_slice(&self.amount.to_le_bytes());
        Ok(())
    }
}