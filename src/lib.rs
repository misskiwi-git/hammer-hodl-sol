use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{clock::Clock, Sysvar},
    program_pack::{Pack, Sealed, IsInitialized},
};

/// 컨트랙트 데이터 구조체
#[derive(Clone, Debug, Default)]
pub struct Vault {
    // 컨트랙트 생성자 (출금 권한자)
    pub owner: Pubkey,
    // 잠금 해제 시간 (Unix timestamp)
    pub unlock_time: i64,
    // 보관된 SOL 양 (lamports 단위)
    pub balance: u64,
    // 초기화 여부
    pub is_initialized: bool,
}

impl Sealed for Vault {}
impl IsInitialized for Vault {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for Vault {
    const LEN: usize = 32 + 8 + 8 + 1; // Pubkey(32) + i64(8) + u64(8) + bool(1)

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let mut offset = 0;
        dst[offset..offset + 32].copy_from_slice(self.owner.as_ref());
        offset += 32;
        dst[offset..offset + 8].copy_from_slice(&self.unlock_time.to_le_bytes());
        offset += 8;
        dst[offset..offset + 8].copy_from_slice(&self.balance.to_le_bytes());
        offset += 8;
        dst[offset] = self.is_initialized as u8;
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if src.len() < Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        let owner = Pubkey::new_from_array(src[0..32].try_into().unwrap());
        let unlock_time = i64::from_le_bytes(src[32..40].try_into().unwrap());
        let balance = u64::from_le_bytes(src[40..48].try_into().unwrap());
        let is_initialized = src[48] != 0;
        Ok(Vault {
            owner,
            unlock_time,
            balance,
            is_initialized,
        })
    }
}

// 프로그램 엔트리포인트 정의
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let vault_account = next_account_info(accounts_iter)?;
    let user_account = next_account_info(accounts_iter)?;
    let clock = Clock::get()?;

    match instruction_data[0] {
        // 초기화 (0)
        0 => {
            // 새 계정은 빈 데이터로 시작하므로 길이 확인 후 Default 사용
            let mut vault_data = if vault_account.data.borrow().len() == Vault::LEN {
                Vault::unpack(&vault_account.data.borrow())?
            } else {
                Vault::default()
            };
            if vault_data.is_initialized() {
                return Err(ProgramError::AccountAlreadyInitialized);
            }
            
            vault_data.owner = *user_account.key;
            vault_data.unlock_time = 1764182400; // 2027-12-24 UTC 타임스탬프
            vault_data.balance = 0;
            vault_data.is_initialized = true;
            
            Vault::pack(vault_data, &mut vault_account.data.borrow_mut())?;
            msg!("Vault initialized with fixed owner and unlock time");
        },

        // 입금 (1)
        1 => {
            let vault_data = Vault::unpack(&vault_account.data.borrow())?;
            if !vault_data.is_initialized() {
                return Err(ProgramError::UninitializedAccount);
            }
            
            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());
            **vault_account.lamports.borrow_mut() += amount;
            **user_account.lamports.borrow_mut() -= amount;
            
            let mut new_vault_data = vault_data.clone();
            new_vault_data.balance += amount;
            Vault::pack(new_vault_data, &mut vault_account.data.borrow_mut())?;
            
            msg!("Deposited {} lamports", amount);
        },

        // 출금 (2)
        2 => {
            let vault_data = Vault::unpack(&vault_account.data.borrow())?;
            if !vault_data.is_initialized() {
                return Err(ProgramError::UninitializedAccount);
            }

            let amount = u64::from_le_bytes(instruction_data[1..9].try_into().unwrap());

            if !user_account.is_signer {
                return Err(ProgramError::MissingRequiredSignature);
            }
            if *user_account.key != vault_data.owner {
                return Err(ProgramError::InvalidAccountData);
            }
            if clock.unix_timestamp < vault_data.unlock_time {
                return Err(ProgramError::Custom(100)); // "Too early to withdraw"
            }
            if amount > vault_data.balance {
                return Err(ProgramError::InsufficientFunds);
            }

            **vault_account.lamports.borrow_mut() -= amount;
            **user_account.lamports.borrow_mut() += amount;
            
            let mut new_vault_data = vault_data.clone();
            new_vault_data.balance -= amount;
            Vault::pack(new_vault_data, &mut vault_account.data.borrow_mut())?;
            
            msg!("Withdrawn {} lamports", amount);
        },
        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}