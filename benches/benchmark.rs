use std::any::Any;

use criterion::{criterion_group, criterion_main, Criterion};
use ring::hmac::{Key, HMAC_SHA256};

use tlspuffin::agent::AgentName;
use tlspuffin::fuzzer::seeds::*;
use tlspuffin::term::dynamic_function::make_dynamic;
use tlspuffin::tls::fn_impl::fn_hmac256;
use tlspuffin::trace::TraceContext;

fn benchmark_dynamic(c: &mut Criterion) {
    let mut group = c.benchmark_group("op_hmac256");

    group.bench_function("op_hmac256 static", |b| {
        b.iter(|| {
            let key_data = [0; 256];
            let key = Key::new(HMAC_SHA256, &key_data);
            let data = "test".as_bytes().to_vec();
            fn_hmac256(&key, &data)
        })
    });

    group.bench_function("op_hmac256 dyn", |b| {
        b.iter(|| {
            let key_data = [0; 256];
            let key = Key::new(HMAC_SHA256, &key_data);
            let data = "test".as_bytes().to_vec();
            let (_, dynamic_fn) = make_dynamic(&fn_hmac256);
            let args: Vec<Box<dyn Any>> = vec![Box::new(key), Box::new(data)];
            dynamic_fn(&args)
        })
    });

    group.finish()
}

fn benchmark_seeds(c: &mut Criterion) {
    let mut group = c.benchmark_group("seeds");

    group.bench_function("seed_successful", |b| {
        b.iter(|| {
            let mut ctx = TraceContext::new();
            let client = AgentName::first();
            let server = client.next();
            let trace = seed_successful(client, server);

            trace.spawn_agents(&mut ctx).unwrap();
            trace.execute(&mut ctx).unwrap();
        })
    });

    group.bench_function("seed_successful12", |b| {
        b.iter(|| {
            let mut ctx = TraceContext::new();
            let client = AgentName::first();
            let server = client.next();
            let trace = seed_successful12(client, server);

            trace.spawn_agents(&mut ctx).unwrap();
            trace.execute(&mut ctx).unwrap()
        })
    });

    group.bench_function("seed_client_attacker", |b| {
        b.iter(|| {
            let mut ctx = TraceContext::new();
            let client = AgentName::first();
            let server = client.next();
            let trace = seed_client_attacker(client, server);

            trace.spawn_agents(&mut ctx).unwrap();
            trace.execute(&mut ctx).unwrap();
        })
    });

    group.bench_function("seed_client_attacker12", |b| {
        b.iter(|| {
            let mut ctx = TraceContext::new();
            let client = AgentName::first();
            let server = client.next();
            let trace = seed_client_attacker12(client, server);

            trace.spawn_agents(&mut ctx).unwrap();
            trace.execute(&mut ctx).unwrap();
        })
    });

    group.finish()
}

criterion_group!(benches, benchmark_dynamic, benchmark_seeds);
criterion_main!(benches);
