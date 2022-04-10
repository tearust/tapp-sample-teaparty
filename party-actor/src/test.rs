#![allow(dead_code)]
#![allow(unused_imports)]
#![feature(const_fn_trait_bound)]


#[cfg(all(test, feature = "__test"))]
mod tests {
	// use crate::*;
	use tea_actor_utility::test_suite::TestSuite;
	use tea_actor_utility::wascc_call;

  fn utility_methods() -> anyhow::Result<()> {
    info!("test crate::utility methods");
    let r1 = utility::uuid_cb_key("uuid", "test");
    assert_eq!(r1, "test_msg_uuid");

    let r2 = utility::cb_key_to_uuid(&r1, "test");
    assert_eq!(r2, "uuid");

    Ok(())
  }

  fn help_methods() -> anyhow::Result<()> {
    info!("test crate::help methods");

    let key = "test_key";
    let val = b"hello".to_vec();
    help::set_mem_cache(&key, val.clone())?;

    assert_eq!(val, help::get_mem_cache(&key)?);

    // has cc link error.

    Ok(())
  }

	#[test]
	fn utility_methods_work() -> anyhow::Result<()> {
		TestSuite::default().run(|_host| {
			
      // utility_methods()?;
      // help_methods()?;

			Ok(())
		})
	}
}