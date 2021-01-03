// {"count":2,"users":[
//  {"id":1,"name":"Sake","email":null,"balance":0,"isActive":false,"isDisabled":false,"created":"2020-12-28 22:49:11","updated":null},
//  {"id":2,"name":"legion","email":null,"balance":0,"isActive":false,"isDisabled":false,"created":"2020-12-28 22:49:23","updated":null}
// ]}

@decco.decode
type user = {
  id: int,
  name: string,
  email: option<string>,
  balance: ValueUtils.balance,
  @decco.key("isActive")
  active: bool,
  @decco.key("isDisabled")
  disabled: bool,
  created: string,
  updated: option<string>,
}

@decco.decode
type users = {users: array<user>}

@decco.decode
type article = {id: int}

@decco.decode
type transaction = {
  id: int,
  quantity: option<int>,
  comment: option<string>,
  amount: int,
  deleted: bool,
  created: string,
  user: user,
  article: option<article>,
  recipient: option<user>,
  sender: option<user>,
}

@decco.encode
type transactionReq = {amount: ValueUtils.balance}

let balanceUser = (uid, amount) => {
  let req = transactionReq_encode({amount: amount})

  open Fetch
  open Js.Promise

  let p = fetchWithInit(
    Consts.userTxApi(uid),
    RequestInit.make(
      ~method_=Post,
      ~headers=Fetch.HeadersInit.make({"Content-Type": "application/json"}),
      ~body=Fetch.BodyInit.make(Js.Json.stringify(req)),
      (),
    ),
  )
  ->then_(Response.text, _)
  ->then_(res => {
    Js.log(res)
    resolve()
  }, _);
  ()
}
