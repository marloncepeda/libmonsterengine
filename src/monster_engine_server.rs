use hyper::{Body, Request, Response, Server};
use hyper::service::service_fn_ok;
use hyper::rt::{self, Future};
use monster_engine_config::MonsterEngineConfig;
use plamo::{PlamoApp, plamo_app_execute, PlamoHttpMethod, PlamoScheme, PlamoByteArray, plamo_request_new, plamo_byte_array_new, plamo_byte_array_get_body_size, plamo_byte_array_get_body};
use std::ffi::CString;
use std::ptr::NonNull;
use std::slice;
use std::sync::Arc;

#[repr(C)]
pub struct MonsterEngineServer {
    app: *const PlamoApp,
    config: *const MonsterEngineConfig,
}

struct MonsterEngineServerWrapper(NonNull<MonsterEngineServer>);

unsafe impl Send for MonsterEngineServerWrapper {}
unsafe impl Sync for MonsterEngineServerWrapper {}

#[no_mangle]
pub extern fn monster_engine_server_new(app: *const PlamoApp, config: *const MonsterEngineConfig) -> *mut MonsterEngineServer {
    Box::into_raw(Box::new(MonsterEngineServer {
        app: app,
        config: config,
    }))
}

#[no_mangle]
pub extern fn monster_engine_server_start(monster_engine_server: *mut MonsterEngineServer) {
    let addr = unsafe { (*(*monster_engine_server).config).bind.to_str().unwrap().parse().unwrap() };
    let monster_engine_server_wrapper = Arc::new(MonsterEngineServerWrapper(NonNull::new(monster_engine_server).unwrap()));

    let server = Server::bind(&addr)
        .serve(move || {
            let monster_engine_server_wrapper = Arc::clone(&monster_engine_server_wrapper);
            service_fn_ok(move |request: Request<Body>| {
                let path = CString::new(request.uri().path()).unwrap();
                let version = CString::new("1.1").unwrap();
                let plamo_byte_array = unsafe { plamo_byte_array_new(std::ptr::null(), 0) };
                let plamo_request = unsafe { plamo_request_new(PlamoHttpMethod::Get, PlamoScheme::Http, path.as_ptr(), version.as_ptr(), plamo_byte_array) };
                let monster_engine_server_ptr = unsafe { (*monster_engine_server_wrapper).0.as_ref() };
                let plamo_response = unsafe { plamo_app_execute(monster_engine_server_ptr.app, plamo_request) };
                Response::new(Body::from(unsafe { slice::from_raw_parts(plamo_byte_array_get_body((*plamo_response).body), plamo_byte_array_get_body_size((*plamo_response).body)) }))
            })
        })
        .map_err(|e| eprintln!("server error: {}", e));

    rt::run(server);
}
