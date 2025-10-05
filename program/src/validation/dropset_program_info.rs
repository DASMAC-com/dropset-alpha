use dropset_interface::error::DropsetError;
use pinocchio::{account_info::AccountInfo, pubkey::pubkey_eq};

#[derive(Clone)]
pub struct _DropsetProgramInfo<'a> {
    pub info: &'a AccountInfo,
}

impl<'a> _DropsetProgramInfo<'a> {
    #[inline(always)]
    pub fn _new(info: &'a AccountInfo) -> Result<_DropsetProgramInfo<'a>, DropsetError> {
        if !pubkey_eq(info.key(), &crate::ID) {
            return Err(DropsetError::IncorrectDropsetProgram);
        }
        Ok(Self { info })
    }
}
