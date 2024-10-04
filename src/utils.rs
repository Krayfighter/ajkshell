


pub fn as_os_str<'a>(source: &'a [u8]) -> &'a std::ffi::OsStr {
  return unsafe { std::mem::transmute(
    source.as_ref()
  ) }
}

// NOTE this may cause undefined behavior
// if there are strange (maybe unicode) contents
// in the source slice
pub fn as_str<'a>(source: &'a [u8]) -> &'a str {
  return unsafe { std::mem::transmute( source ) }
}



