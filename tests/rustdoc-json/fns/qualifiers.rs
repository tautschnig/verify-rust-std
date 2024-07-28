//@ edition:2018

//@ is "$.index[*][?(@.name=='nothing_fn')].inner.function.header.async" false
//@ is "$.index[*][?(@.name=='nothing_fn')].inner.function.header.const"  false
//@ is "$.index[*][?(@.name=='nothing_fn')].inner.function.header.unsafe" false
pub fn nothing_fn() {}

//@ is "$.index[*][?(@.name=='unsafe_fn')].inner.function.header.async"  false
//@ is "$.index[*][?(@.name=='unsafe_fn')].inner.function.header.const"  false
//@ is "$.index[*][?(@.name=='unsafe_fn')].inner.function.header.unsafe" true
pub unsafe fn unsafe_fn() {}

//@ is "$.index[*][?(@.name=='const_fn')].inner.function.header.async"  false
//@ is "$.index[*][?(@.name=='const_fn')].inner.function.header.const"  true
//@ is "$.index[*][?(@.name=='const_fn')].inner.function.header.unsafe" false
pub const fn const_fn() {}

//@ is "$.index[*][?(@.name=='async_fn')].inner.function.header.async"  true
//@ is "$.index[*][?(@.name=='async_fn')].inner.function.header.const"  false
//@ is "$.index[*][?(@.name=='async_fn')].inner.function.header.unsafe" false
pub async fn async_fn() {}

//@ is "$.index[*][?(@.name=='async_unsafe_fn')].inner.function.header.async"  true
//@ is "$.index[*][?(@.name=='async_unsafe_fn')].inner.function.header.const"  false
//@ is "$.index[*][?(@.name=='async_unsafe_fn')].inner.function.header.unsafe" true
pub async unsafe fn async_unsafe_fn() {}

//@ is "$.index[*][?(@.name=='const_unsafe_fn')].inner.function.header.async"  false
//@ is "$.index[*][?(@.name=='const_unsafe_fn')].inner.function.header.const"  true
//@ is "$.index[*][?(@.name=='const_unsafe_fn')].inner.function.header.unsafe" true
pub const unsafe fn const_unsafe_fn() {}

// It's impossible for a function to be both const and async, so no test for that
