use cilk::{
    exec::interpreter::interp,
    ir::{builder, function, module, opcode, types, value},
};

#[test]
fn interpret_phi() {
    let mut m = module::Module::new("cilk");

    let f_id = m.add_function(function::Function::new(
        "f",
        types::Type::Int32,
        vec![types::Type::Int32],
    ));
    let mut builder = builder::Builder::new(&mut m, f_id);

    let bb = builder.append_basic_block();
    let bb2 = builder.append_basic_block();
    let if_true = builder.append_basic_block();
    let if_false = builder.append_basic_block();
    let merge = builder.append_basic_block();

    builder.set_insert_point(bb);
    let var = builder.build_alloca(types::Type::Int32);
    let val = builder.build_load(var);
    let val2 = builder.build_add(
        val,
        value::Value::Immediate(value::ImmediateValue::Int32(1)),
    );
    builder.build_br(bb2);
    builder.set_insert_point(bb2);
    let arg0 = builder.get_param(0).unwrap();
    let val3 = builder.build_add(val2, arg0);
    let eq = builder.build_icmp(
        opcode::ICmpKind::Eq,
        val3,
        value::Value::Immediate(value::ImmediateValue::Int32(4)),
    );
    builder.build_cond_br(eq, if_true, if_false);
    builder.set_insert_point(if_true);
    builder.build_br(merge);
    builder.set_insert_point(if_false);
    builder.build_br(merge);
    builder.set_insert_point(merge);
    let ret = builder.build_phi(vec![
        (
            value::Value::Immediate(value::ImmediateValue::Int32(1)),
            if_true,
        ),
        (val3, if_false),
    ]);
    builder.build_ret(ret);

    let f = m.function_ref(f_id);
    println!("{}", f.to_string(&m));

    let mut interp = interp::Interpreter::new(&m);
    let ret = interp.run_function(f_id, vec![interp::ConcreteValue::Int32(3)]);
    assert_eq!(ret, interp::ConcreteValue::Int32(1));
    let ret = interp.run_function(f_id, vec![interp::ConcreteValue::Int32(5)]);
    assert_eq!(ret, interp::ConcreteValue::Int32(6));

    println!("exec: f(5) = {:?}", ret);
}
