pub fn runner(tests: &[&test::TestDescAndFn]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        print!("{}...", test.desc.name);
        match test.testfn {
            test::TestFn::StaticTestFn(f) => f(),
            test::TestFn::StaticBenchFn(_) => todo!(),
            test::TestFn::DynTestFn(_) => todo!(),
            test::TestFn::DynBenchFn(_) => todo!(),
        }
        println!("[ok]");
    }
}

