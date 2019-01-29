// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702
#![allow(unknown_lints)]
#![allow(clippy)]

#![cfg_attr(rustfmt, rustfmt_skip)]

#![allow(box_pointers)]
#![allow(dead_code)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(trivial_casts)]
#![allow(unsafe_code)]
#![allow(unused_imports)]
#![allow(unused_results)]


// interface

pub trait Ring {
    fn start_roll_call(&self, o: ::grpc::RequestOptions, p: super::ring::Empty) -> ::grpc::SingleResponse<super::ring::Empty>;

    fn mark_itself(&self, o: ::grpc::RequestOptions, p: super::ring::Empty) -> ::grpc::SingleResponse<super::ring::Empty>;
}

// client

pub struct RingClient {
    grpc_client: ::std::sync::Arc<::grpc::Client>,
    method_StartRollCall: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::ring::Empty, super::ring::Empty>>,
    method_MarkItself: ::std::sync::Arc<::grpc::rt::MethodDescriptor<super::ring::Empty, super::ring::Empty>>,
}

impl ::grpc::ClientStub for RingClient {
    fn with_client(grpc_client: ::std::sync::Arc<::grpc::Client>) -> Self {
        RingClient {
            grpc_client: grpc_client,
            method_StartRollCall: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/ringproto.Ring/StartRollCall".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
            method_MarkItself: ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                name: "/ringproto.Ring/MarkItself".to_string(),
                streaming: ::grpc::rt::GrpcStreaming::Unary,
                req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
            }),
        }
    }
}

impl Ring for RingClient {
    fn start_roll_call(&self, o: ::grpc::RequestOptions, p: super::ring::Empty) -> ::grpc::SingleResponse<super::ring::Empty> {
        self.grpc_client.call_unary(o, p, self.method_StartRollCall.clone())
    }

    fn mark_itself(&self, o: ::grpc::RequestOptions, p: super::ring::Empty) -> ::grpc::SingleResponse<super::ring::Empty> {
        self.grpc_client.call_unary(o, p, self.method_MarkItself.clone())
    }
}

// server

pub struct RingServer;


impl RingServer {
    pub fn new_service_def<H : Ring + 'static + Sync + Send + 'static>(handler: H) -> ::grpc::rt::ServerServiceDefinition {
        let handler_arc = ::std::sync::Arc::new(handler);
        ::grpc::rt::ServerServiceDefinition::new("/ringproto.Ring",
            vec![
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/ringproto.Ring/StartRollCall".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.start_roll_call(o, p))
                    },
                ),
                ::grpc::rt::ServerMethod::new(
                    ::std::sync::Arc::new(::grpc::rt::MethodDescriptor {
                        name: "/ringproto.Ring/MarkItself".to_string(),
                        streaming: ::grpc::rt::GrpcStreaming::Unary,
                        req_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                        resp_marshaller: Box::new(::grpc::protobuf::MarshallerProtobuf),
                    }),
                    {
                        let handler_copy = handler_arc.clone();
                        ::grpc::rt::MethodHandlerUnary::new(move |o, p| handler_copy.mark_itself(o, p))
                    },
                ),
            ],
        )
    }
}
