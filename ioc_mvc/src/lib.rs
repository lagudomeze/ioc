pub use poem_openapi::{OpenApi, OpenApiService};

pub use ioc_mvc_derive::mvc;
pub use server::{run_mvc, WebConfig};

mod server;

pub trait OpenApiExt: OpenApi {
    fn join<T>(self, api: T) -> (Self, T) {
        (self, api)
    }
}

impl<T> OpenApiExt for T
where
    T: OpenApi,
{}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use std::collections::HashMap;

    use poem_openapi::__private::poem::endpoint::BoxEndpoint;
    use poem_openapi::__private::poem::http::Method;
    use poem_openapi::OpenApi;
    use poem_openapi::registry::{MetaApi, Registry};

    use crate::OpenApiExt;

    struct A;
    struct B;
    struct C;

    impl OpenApi for A {
        fn meta() -> Vec<MetaApi> {
            unimplemented!()
        }

        fn register(registry: &mut Registry) {
            unimplemented!()
        }

        fn add_routes(self, route_table: &mut HashMap<String, HashMap<Method, BoxEndpoint<'static>>>) {
            unimplemented!()
        }
    }

    impl OpenApi for B {
        fn meta() -> Vec<MetaApi> {
            unimplemented!()
        }

        fn register(registry: &mut Registry) {
            unimplemented!()
        }

        fn add_routes(self, route_table: &mut HashMap<String, HashMap<Method, BoxEndpoint<'static>>>) {
            unimplemented!()
        }
    }

    impl OpenApi for C {
        fn meta() -> Vec<MetaApi> {
            unimplemented!()
        }

        fn register(registry: &mut Registry) {
            unimplemented!()
        }

        fn add_routes(self, route_table: &mut HashMap<String, HashMap<Method, BoxEndpoint<'static>>>) {
            unimplemented!()
        }
    }

    fn test2<T: OpenApiExt>(_: T) {}

    #[test]
    fn it_works() {
        let i = A;
        let i = i.join(B);
        let i = i.join(C);
        test2(i);
    }
}
