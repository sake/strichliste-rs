type fetchError =
  | HttpError
  | DecodeError

type fetchState<'a> =
  | InitialState
  | Loading
  | Failed(fetchError)
  | Fetched('a)

exception DecodeException(Decco.decodeError)

let unwrapDecode = (result): 'a => {
  switch result {
  | Ok(data) => data
  | Error(e) => raise(DecodeException(e))
  }
}

type dataCallbackFn<'a> = fetchState<'a> => unit
type decoderFn<'a> = Js.Json.t => Belt.Result.t<'a, Decco.decodeError>
type transformerFn<'a, 'b> = 'a => 'b

let identity = v => v

let fetchTemplate = (
  url,
  decoder: decoderFn<'a>,
  transformer: transformerFn<'a, 'b>,
  callback: dataCallbackFn<'b>,
) => {
  open Js.Promise
  Fetch.fetch(url)
    ->then_(Fetch.Response.json, _)
    ->then_(json => {
        decoder(json)->unwrapDecode->transformer->resolve
      }, _)
    ->then_(data => {
        callback(Fetched(data))
        resolve()
      }, _)
    ->ignore
}
