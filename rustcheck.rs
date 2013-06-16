
extern mod std;
extern mod extra;

use std::rand::RngUtil;

trait Arbitrary {
  fn arbitrary(test_context: &mut TestContext) -> Self;
}

trait Shrink {
  fn shrink(value: &Self) -> ~[Self];
}

impl Arbitrary for int {
  fn arbitrary(test_context: &mut TestContext) -> int {
    choose(test_context, 0, test_context.test_iteration_size)
  }
}

impl Arbitrary for bool {
  fn arbitrary(test_context: &mut TestContext) -> bool {
    let value = choose(test_context, 0, 1);
    value == 1
  }
}

impl<T: Arbitrary> Arbitrary for ~[T] {
  fn arbitrary(test_context: &mut TestContext) -> ~[T] {
    let sz = choose(test_context, 0, test_context.test_iteration_size) as uint;
    do std::vec::build_sized(sz) |push| {
      for sz.times {
        push(Arbitrary::arbitrary(test_context))
      }
    }
  }
}

fn choose(test_context: &mut TestContext, start: int, end: int) -> int {
  let rnd = &mut test_context.test_random_generator;
  rnd.gen_int_range(start, end+1)
}

enum TestResult {
  Success,
  ExhaustedGenerators(uint),
  Fail(@extra::list::List<~str>)
}

struct TestContext {
  test_random_generator: std::rand::IsaacRng,
  test_iteration_size: int
}

fn for_all<T: Arbitrary + ToStr>(test_context: &mut TestContext, op: &fn(arg: &T) -> TestResult) -> TestResult {
  let arg = &Arbitrary::arbitrary(test_context);
  let result = op(arg);
  match result {
    Success => Success,
    ExhaustedGenerators(tries) => ExhaustedGenerators(tries),
    Fail(tries) => Fail(@extra::list::Cons(arg.to_str(), tries))
  }
}

enum Test {
  Test(~str, ~fn(&mut TestContext) -> TestResult)
}

fn run_test(test: &Test) {
  // let &Test(ref test_name, _) = test; // issue #4653
  let test_name = (match test { &Test(ref test_name, _) => test_name }).to_owned();
  match run_a_test(test) {
    Success => std::io::println(fmt!("success %s", test_name)),
    Fail(ref strs) => std::io::println(fmt!("failed %s, counter example:\n%?", test_name, *strs)),
    ExhaustedGenerators(tries) => std::io::println(fmt!("exhausted generators after %u tries", tries))
  }
}

fn run_a_test(test: &Test) -> TestResult {
  //  let &Test(_, ref prop_fun) = test; // issue #4653
  let prop_fun = match test { &Test(_, ref prop_fun) => prop_fun };
  let mut test_context = TestContext{
    test_random_generator: std::rand::rng(),
    test_iteration_size: 0
  };
  for [10, ..1000].each |iteration_size| {
    test_context.test_iteration_size = *iteration_size;
    let prop = (*prop_fun)(&mut test_context);
    match prop {
      Success => (),
      Fail(strs) => return Fail(strs),
      ExhaustedGenerators(tries) => return ExhaustedGenerators(tries)
    };
  }
  Success
}

fn run_tests(tests: &[Test]) {
  for tests.each |test| {
    run_test(test);
  }
}

trait TestResultish {
  fn to_test_result(value: &Self) -> TestResult;
}

impl TestResultish for bool {
  fn to_test_result(value: &bool) -> TestResult {
    if *value {
      Success
    } else {
      Fail(@extra::list::Nil)
    }
  }
}

#[main]
fn main() {
  let p = Test(~"reverse vec", |tctx|
    do for_all(tctx) |arg: &~[int]| {
      let mut reversed = std::vec::reversed(*arg);
      reversed[0] = 0;
      let double_reveresed = std::vec::reversed(reversed);
      TestResultish::to_test_result(&(*arg == double_reveresed))
    });
  run_tests([p]);
}