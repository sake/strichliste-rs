@decco.decode
type user = {user: Service.user}

let fetchUser = (uid, callback) => {
  FetchUtils.fetchTemplate(Consts.userApi(uid), user_decode, user => user.user, callback)
}

@react.component
let make = () => {
  let router = Next.Router.useRouter()
  let query = router.query
  let uid = Js.Dict.get(query, "uid")

  let (state, setState) = React.useState(() => FetchUtils.InitialState)
  React.useEffect1(() => {
    switch uid {
    | Some(u) => fetchUser(u, data => setState(_state => data))
    | None => ()
    }

    None
  }, [uid])

  <Root>
    <div>
      <h1> {React.string("User!")} </h1>
      {switch state {
      | InitialState
      | Loading =>
        <p> {React.string("loading ...")} </p>
      | Fetched(user) => <User user />
      | Failed(error) => <Error error />
      }}
    </div>
  </Root>
}

let default = make
