use cilk::{
    exec::{interpreter::interp, jit::x64::compiler},
    *,
};
use rustc_hash::FxHashMap;

#[test]
pub fn jit_prime() {
    let mut m = module::Module::new("cilk");

    // Internal function must be defined when you use it
    let cilk_println_i32 = m.add_function(ir::function::Function::new(
        "cilk.println.i32",
        ir::types::Type::Void,
        vec![ir::types::Type::Int32],
    ));

    /*
     * int prime(int n) {
     *     if (n == 2) return 1;
     *     if (n % 2 == 0) return 0;
     *     for (int i = 3; i * i <= n; i += 2) {
     *         if (n % i == 0) return 0;
     *     }
     *     return 1;
     * }
     */
    let prime = cilk_ir!(m; define [i32] prime (i32) {
        entry:
            i = alloca i32;
            cond = icmp eq (%arg.0), (i32 2);
            br (%cond) l1, l2;
        l1:
            ret (i32 1);
        l2:
            r = rem (%arg.0), (i32 2);
            cond = icmp eq (%r), (i32 0);
            br (%cond) l3, l4;
        l3:
            ret (i32 0);
        l4:
            store (i32 3), (%i);
            br l5;
        l5:
            li = load (%i);
            m = mul (%li), (%li);
            cond = icmp le (%m), (%arg.0);
            br (%cond) l6, l7;
        l6:
            li = load (%i);
            r = rem (%arg.0), (%li);
            cond = icmp eq (%r), (i32 0);
            br (%cond) l8, l9;
        l8:
            ret (i32 0);
        l9:
            a = add (%li), (i32 2);
            store (%a), (%i);
            br l5;
        l7:
            ret (i32 1);
    });

    let main = cilk_ir!(m; define [void] main () {
        entry:
            i = alloca i32;
            store (i32 2), (%i);
            br l1;
        l1:
            li = load (%i);
            c = icmp le (%li), (i32 100);
            br (%c) l2, l3;
        l2:
            p = call prime [(%li)];
            c = icmp eq (%p), (i32 1);
            br (%c) l4, l5;
        l4:
            __ = call (->cilk_println_i32) [(%li)];
            br l5;
        l5:
            a = add (%li), (i32 1);
            store (%a), (%i);
            br l1;
        l3:
            ret (void);
    });

    let mut interp = interp::Interpreter::new(&m);

    let ret = interp.run_function(prime, vec![interp::ConcreteValue::Int32(97)]);
    println!("interp: prime(97) = {:?}", ret);
    assert_eq!(ret, interp::ConcreteValue::Int32(1));

    let ret = interp.run_function(prime, vec![interp::ConcreteValue::Int32(104)]);
    println!("interp: prime(104) = {:?}", ret);
    assert_eq!(ret, interp::ConcreteValue::Int32(0));

    let mut jit = compiler::JITCompiler::new(&m);
    jit.compile_module();

    println!("liveness: {}", m.function_ref(prime).to_string(&m));
    println!("liveness: {}", m.function_ref(main).to_string(&m));

    let ret = jit.run(prime, vec![compiler::GenericValue::Int32(10009723)]);
    println!("jit: prime(10009723) = {:?}", ret);
    assert_eq!(ret, compiler::GenericValue::Int32(1));

    let ret = jit.run(prime, vec![compiler::GenericValue::Int32(10009721)]);
    println!("jit: prime(10009721) = {:?}", ret);
    assert_eq!(ret, compiler::GenericValue::Int32(0));

    println!("main:");
    jit.run(main, vec![]);
}
