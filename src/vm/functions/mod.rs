pub mod define;
pub mod tuples;
mod iterables;
mod arithmetic;
mod boolean;
mod database;
mod options;
mod assets;

use vm::errors::{CheckErrors, RuntimeErrorType, ShortReturnType, InterpreterResult as Result, check_argument_count, check_arguments_at_least};
use vm::types::{Value, PrincipalData, ResponseData, TypeSignature};
use vm::callables::{CallableType, NativeHandle};
use vm::representations::{SymbolicExpression, SymbolicExpressionType, ClarityName};
use vm::representations::SymbolicExpressionType::{List, Atom};
use vm::{LocalContext, Environment, eval};
use vm::costs::cost_functions;
use util::hash;

define_named_enum!(NativeFunctions {
    Add("+"),
    Subtract("-"),
    Multiply("*"),
    Divide("/"),
    CmpGeq(">="),
    CmpLeq("<="),
    CmpLess("<"),
    CmpGreater(">"),
    ToInt("to-int"),
    ToUInt("to-uint"),
    Modulo("mod"),
    Power("pow"),
    BitwiseXOR("xor"),
    And("and"),
    Or("or"),
    Not("not"),
    Equals("is-eq"),
    If("if"),
    Let("let"),
    Map("map"),
    Fold("fold"),
    Append("append"),
    Concat("concat"),
    AsMaxLen("as-max-len?"),
    Len("len"),
    ListCons("list"),
    FetchVar("var-get"),
    SetVar("var-set"),
    FetchEntry("map-get?"),
    FetchContractEntry("contract-map-get?"),
    SetEntry("map-set"),
    InsertEntry("map-insert"),
    DeleteEntry("map-delete"),
    TupleCons("tuple"),
    TupleGet("get"),
    Begin("begin"),
    Hash160("hash160"),
    Sha256("sha256"),
    Sha512("sha512"),
    Sha512Trunc256("sha512/256"),
    Keccak256("keccak256"),
    Print("print"),
    ContractCall("contract-call?"),
    AsContract("as-contract"),
    AtBlock("at-block"),
    GetBlockInfo("get-block-info?"),
    ConsError("err"),
    ConsOkay("ok"),
    ConsSome("some"),
    DefaultTo("default-to"),
    Asserts("asserts!"),
    UnwrapRet("unwrap!"),
    UnwrapErrRet("unwrap-err!"),
    Unwrap("unwrap-panic"),
    UnwrapErr("unwrap-err-panic"),
    Match("match"),
    TryRet("try!"),
    IsOkay("is-ok"),
    IsNone("is-none"),
    IsErr("is-err"),
    IsSome("is-some"),
    Filter("filter"),
    GetTokenBalance("ft-get-balance"),
    GetAssetOwner("nft-get-owner?"),
    TransferToken("ft-transfer?"),
    TransferAsset("nft-transfer?"),
    MintAsset("nft-mint?"),
    MintToken("ft-mint?"),
    StxTransfer("stx-transfer?"),
    StxBurn("stx-burn?"),
});

pub fn lookup_reserved_functions(name: &str) -> Option<CallableType> {
    use vm::functions::NativeFunctions::*;
    use vm::callables::CallableType::{ NativeFunction, SpecialFunction };
    if let Some(native_function) = NativeFunctions::lookup_by_name(name) {
        let callable = match native_function {
            Add => NativeFunction("native_add", NativeHandle::MoreArg(&arithmetic::native_add), cost_functions::ADD),
            Subtract => NativeFunction("native_sub", NativeHandle::MoreArg(&arithmetic::native_sub), cost_functions::SUB),
            Multiply => NativeFunction("native_mul", NativeHandle::MoreArg(&arithmetic::native_mul), cost_functions::MUL),
            Divide => NativeFunction("native_div", NativeHandle::MoreArg(&arithmetic::native_div), cost_functions::DIV),
            CmpGeq => NativeFunction("native_geq", NativeHandle::DoubleArg(&arithmetic::native_geq), cost_functions::GEQ),
            CmpLeq => NativeFunction("native_leq", NativeHandle::DoubleArg(&arithmetic::native_leq), cost_functions::LEQ),
            CmpLess => NativeFunction("native_le", NativeHandle::DoubleArg(&arithmetic::native_le), cost_functions::LE),
            CmpGreater => NativeFunction("native_ge", NativeHandle::DoubleArg(&arithmetic::native_ge), cost_functions::GE),
            ToUInt => NativeFunction("native_to_uint", NativeHandle::SingleArg(&arithmetic::native_to_uint), cost_functions::INT_CAST),
            ToInt => NativeFunction("native_to_int", NativeHandle::SingleArg(&arithmetic::native_to_int), cost_functions::INT_CAST),
            Modulo => NativeFunction("native_mod", NativeHandle::DoubleArg(&arithmetic::native_mod), cost_functions::MOD),
            Power => NativeFunction("native_pow", NativeHandle::DoubleArg(&arithmetic::native_pow), cost_functions::POW),
            BitwiseXOR => NativeFunction("native_xor", NativeHandle::DoubleArg(&arithmetic::native_xor), cost_functions::XOR),
            And => SpecialFunction("special_and", &boolean::special_and),
            Or => SpecialFunction("special_or", &boolean::special_or),
            Not => NativeFunction("native_not", NativeHandle::SingleArg(&boolean::native_not), cost_functions::NOT),
            Equals => NativeFunction("native_eq", NativeHandle::MoreArg(&native_eq), cost_functions::EQ),
            If => SpecialFunction("special_if", &special_if),
            Let => SpecialFunction("special_let", &special_let),
            FetchVar => SpecialFunction("special_var-get", &database::special_fetch_variable),
            SetVar => SpecialFunction("special_set-var", &database::special_set_variable),
            Map => SpecialFunction("special_map", &iterables::special_map),
            Filter => SpecialFunction("special_filter", &iterables::special_filter),
            Fold => SpecialFunction("special_fold", &iterables::special_fold),
            Concat => SpecialFunction("special_concat", &iterables::special_concat),
            AsMaxLen => SpecialFunction("special_as_max_len", &iterables::special_as_max_len),
            Append => SpecialFunction("special_append", &iterables::special_append),
            Len => NativeFunction("native_len", NativeHandle::SingleArg(&iterables::native_len), cost_functions::LEN),
            ListCons => SpecialFunction("special_list_cons", &iterables::list_cons),
            FetchEntry => SpecialFunction("special_map-get?", &database::special_fetch_entry),
            FetchContractEntry => SpecialFunction("special_contract-map-get?", &database::special_fetch_contract_entry),
            SetEntry => SpecialFunction("special_set-entry", &database::special_set_entry),
            InsertEntry => SpecialFunction("special_insert-entry", &database::special_insert_entry),
            DeleteEntry => SpecialFunction("special_delete-entry", &database::special_delete_entry),
            TupleCons => SpecialFunction("special_tuple", &tuples::tuple_cons),
            TupleGet => SpecialFunction("special_get-tuple", &tuples::tuple_get),
            Begin => NativeFunction("native_begin", NativeHandle::MoreArg(&native_begin), cost_functions::BEGIN),
            Hash160 => NativeFunction("native_hash160", NativeHandle::SingleArg(&native_hash160), cost_functions::HASH160),
            Sha256 => NativeFunction("native_sha256", NativeHandle::SingleArg(&native_sha256), cost_functions::SHA256),
            Sha512 => NativeFunction("native_sha512", NativeHandle::SingleArg(&native_sha512), cost_functions::SHA512),
            Sha512Trunc256 => NativeFunction("native_sha512trunc256", NativeHandle::SingleArg(&native_sha512trunc256), cost_functions::SHA512T256),
            Keccak256 => NativeFunction("native_keccak256", NativeHandle::SingleArg(&native_keccak256), cost_functions::KECCAK256),
            Print => NativeFunction("native_print", NativeHandle::SingleArg(&native_print), cost_functions::PRINT),
            ContractCall => SpecialFunction("special_contract-call", &database::special_contract_call),
            AsContract => SpecialFunction("special_as-contract", &special_as_contract),
            GetBlockInfo => SpecialFunction("special_get_block_info", &database::special_get_block_info),
            ConsSome => NativeFunction("native_some", NativeHandle::SingleArg(&options::native_some), cost_functions::SOME_CONS),
            ConsOkay => NativeFunction("native_okay", NativeHandle::SingleArg(&options::native_okay), cost_functions::OK_CONS),
            ConsError => NativeFunction("native_error", NativeHandle::SingleArg(&options::native_error), cost_functions::ERR_CONS),
            DefaultTo => NativeFunction("native_default_to", NativeHandle::DoubleArg(&options::native_default_to), cost_functions::DEFAULT_TO),
            Asserts => SpecialFunction("special_asserts", &special_asserts),
            UnwrapRet => NativeFunction("native_unwrap_ret", NativeHandle::DoubleArg(&options::native_unwrap_or_ret), cost_functions::UNWRAP_RET),
            UnwrapErrRet => NativeFunction("native_unwrap_err_ret", NativeHandle::DoubleArg(&options::native_unwrap_err_or_ret), cost_functions::UNWRAP_ERR_OR_RET),
            IsOkay => NativeFunction("native_is_okay", NativeHandle::SingleArg(&options::native_is_okay), cost_functions::IS_OKAY),
            IsNone => NativeFunction("native_is_none", NativeHandle::SingleArg(&options::native_is_none), cost_functions::IS_NONE),
            IsErr => NativeFunction("native_is_err", NativeHandle::SingleArg(&options::native_is_err), cost_functions::IS_ERR),
            IsSome => NativeFunction("native_is_some", NativeHandle::SingleArg(&options::native_is_some), cost_functions::IS_SOME),
            Unwrap => NativeFunction("native_unwrap", NativeHandle::SingleArg(&options::native_unwrap), cost_functions::UNWRAP),
            UnwrapErr => NativeFunction("native_unwrap_err", NativeHandle::SingleArg(&options::native_unwrap_err), cost_functions::UNWRAP_ERR),
            Match => SpecialFunction("special_match", &options::special_match),
            TryRet => NativeFunction("native_try_ret", NativeHandle::SingleArg(&options::native_try_ret), cost_functions::TRY_RET),
            MintAsset => SpecialFunction("special_mint_asset", &assets::special_mint_asset),
            MintToken => SpecialFunction("special_mint_token", &assets::special_mint_token),
            TransferAsset => SpecialFunction("special_transfer_asset", &assets::special_transfer_asset),
            TransferToken => SpecialFunction("special_transfer_token", &assets::special_transfer_token),
            GetTokenBalance => SpecialFunction("special_get_balance", &assets::special_get_balance),
            GetAssetOwner => SpecialFunction("special_get_owner", &assets::special_get_owner),
            AtBlock => SpecialFunction("special_at_block", &database::special_at_block),
            StxTransfer => SpecialFunction("special_stx_transfer", &assets::special_stx_transfer),
            StxBurn => SpecialFunction("special_stx_burn", &assets::special_stx_burn),
        };
        Some(callable)
    } else {
        None
    }
}

fn native_eq(args: Vec<Value>) -> Result<Value> {
    // TODO: this currently uses the derived equality checks of Value,
    //   however, that's probably not how we want to implement equality
    //   checks on the ::ListTypes

    if args.len() < 2 {
        Ok(Value::Bool(true))
    } else {
        let first = &args[0];
        // check types:
        let mut arg_type = TypeSignature::type_of(first);
        for x in args.iter() {
            arg_type = TypeSignature::least_supertype(&TypeSignature::type_of(x), &arg_type)?;
            if x != first {
                return Ok(Value::Bool(false))
            }
        }
        Ok(Value::Bool(true))
    }
}

macro_rules! native_hash_func {
    ($name:ident, $module:ty) => {
        fn $name(input: Value) -> Result<Value> {
            let bytes = match input {
                Value::Int(value) => Ok(value.to_le_bytes().to_vec()),
                Value::UInt(value) => Ok(value.to_le_bytes().to_vec()),
                Value::Buffer(value) => Ok(value.data),
                _ => Err(CheckErrors::UnionTypeValueError(vec![TypeSignature::IntType, TypeSignature::UIntType, TypeSignature::max_buffer()], input))
            }?;
            let hash = <$module>::from_data(&bytes);
            Value::buff_from(hash.as_bytes().to_vec())
        }
    }
}

native_hash_func!(native_hash160, hash::Hash160);
native_hash_func!(native_sha256, hash::Sha256Sum);
native_hash_func!(native_sha512, hash::Sha512Sum);
native_hash_func!(native_sha512trunc256, hash::Sha512Trunc256Sum);
native_hash_func!(native_keccak256, hash::Keccak256Hash);

fn native_begin(mut args: Vec<Value>) -> Result<Value> {
    match args.pop() {
        Some(v) => Ok(v),
        None => Err(CheckErrors::RequiresAtLeastArguments(1,0).into())
    }
}

fn native_print(input: Value) -> Result<Value> {
    if cfg!(feature = "developer-mode") {
        eprintln!("{}", &input);
    }
    Ok(input)
}

fn special_if(args: &[SymbolicExpression], env: &mut Environment, context: &LocalContext) -> Result<Value> {
    check_argument_count(3, args)?;

    runtime_cost!(cost_functions::IF, env, 0)?;
    // handle the conditional clause.
    let conditional = eval(&args[0], env, context)?;
    match conditional {
        Value::Bool(result) => {
            if result {
                eval(&args[1], env, context)
            } else {
                eval(&args[2], env, context)
            }
        },
        _ => Err(CheckErrors::TypeValueError(TypeSignature::BoolType, conditional).into())
    }
}

fn special_asserts(args: &[SymbolicExpression], env: &mut Environment, context: &LocalContext) -> Result<Value> {
    check_argument_count(2, args)?;

    runtime_cost!(cost_functions::ASSERTS, env, 0)?;
    // handle the conditional clause.
    let conditional = eval(&args[0], env, context)?;

    match conditional {
        Value::Bool(result) => {
            if result {
                Ok(conditional)
            } else {
                let thrown = eval(&args[1], env, context)?;
                Err(ShortReturnType::AssertionFailed(thrown.clone()).into())
            }
        },
        _ => Err(CheckErrors::TypeValueError(TypeSignature::BoolType, conditional).into())
    }
}

pub fn handle_binding_list <F, E> (bindings: &[SymbolicExpression], mut handler: F) -> std::result::Result<(), E>
where F: FnMut(&ClarityName, &SymbolicExpression) -> std::result::Result<(), E>,
      E: From<CheckErrors>
{
    for binding in bindings.iter() {
        let binding_expression = binding.match_list()
            .ok_or(CheckErrors::BadSyntaxBinding)?;
        if binding_expression.len() != 2 {
            return Err(CheckErrors::BadSyntaxBinding.into());
        }
        let var_name = binding_expression[0].match_atom()
            .ok_or(CheckErrors::BadSyntaxBinding)?;
        let var_sexp = &binding_expression[1];

        handler(var_name, var_sexp)?;
    }
    Ok(())
}

pub fn parse_eval_bindings(bindings: &[SymbolicExpression],
                       env: &mut Environment, context: &LocalContext)-> Result<Vec<(ClarityName, Value)>> {
    let mut result = Vec::new();
    handle_binding_list(bindings, |var_name, var_sexp| {
        eval(var_sexp, env, context)
            .and_then(|value| {
                result.push((var_name.clone(), value));
                Ok(()) })
    })?;

    Ok(result)
}

fn special_let(args: &[SymbolicExpression], env: &mut Environment, context: &LocalContext) -> Result<Value> {
    use vm::is_reserved;

    // (let ((x 1) (y 2)) (+ x y)) -> 3
    // arg0 => binding list
    // arg1..n => body
    check_arguments_at_least(2, args)?;

    // parse and eval the bindings.
    let bindings = args[0].match_list()
        .ok_or(CheckErrors::BadLetSyntax)?;

    let mut binding_results = parse_eval_bindings(bindings, env, context)?;

    runtime_cost!(cost_functions::LET, env, binding_results.len())?;

    // create a new context.
    let mut inner_context = context.extend()?;

    for (binding_name, binding_value) in binding_results.drain(..) {
        if is_reserved(&binding_name) ||
           env.contract_context.lookup_function(&binding_name).is_some() ||
           inner_context.lookup_variable(&binding_name).is_some() {
            return Err(CheckErrors::NameAlreadyUsed(binding_name.into()).into())
        }
        inner_context.variables.insert(binding_name, binding_value);
    }

    // evaluate the let-bodies

    let mut last_result = None;
    for body in args[1..].iter() {
        let body_result = eval(&body, env, &inner_context)?;
        last_result.replace(body_result);
    }

    // last_result should always be Some(...), because of the arg len check above.
    Ok(last_result.unwrap())
}

fn special_as_contract(args: &[SymbolicExpression], env: &mut Environment, context: &LocalContext) -> Result<Value> {
    // (as-contract (..))
    // arg0 => body
    check_argument_count(1, args)?;

    // nest an environment.
    let contract_principal = Value::Principal(PrincipalData::Contract(env.contract_context.contract_identifier.clone()));
    let mut nested_env = env.nest_as_principal(contract_principal);

    eval(&args[0], &mut nested_env, context)
}