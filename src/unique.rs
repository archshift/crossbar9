#[derive(Debug)]
pub enum Error {
	AlreadyLocked,
}

pub fn lock(var: &mut bool) -> Result<(), Error> {
    if *var {
        return Err(Error::AlreadyLocked)
    }
    *var = true;
    return Ok(())
}

pub fn unlock(var: &mut bool) {
    *var = false;
}
