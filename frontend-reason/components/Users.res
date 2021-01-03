let fetchUsers = callback => {
  FetchUtils.fetchTemplate(Consts.usersApi(), Service.users_decode, users => users.users, callback)
}

type state = FetchUtils.fetchState<Service.users>

@react.component
let make = () => {
  open FetchUtils

  let settings = React.useContext(Settings.settingsCtx)

  let (state, setState) = React.useState(() => InitialState)
  React.useEffect0(() => {
    fetchUsers(data => setState(_state => data))
    None
  })

  <div>
    <h1> {React.string("Users!")} </h1>
    {switch state {
    | InitialState
    | Loading =>
      <p> {React.string("loading ...")} </p>
    | Fetched(users) =>
      Array.map(
        (u: Service.user) =>
          <Next.Link key={u.id->Belt.Int.toString} href={j`/user/${u.id->Belt.Int.toString}`}>
            <a>
              <div>
                <p> {React.string(u.name)} </p>
                <p> {React.string(ValueUtils.formatBalance(settings, u.balance))} </p>
              </div>
            </a>
          </Next.Link>,
        users,
      )->React.array
    | Failed(error) => <Error error />
    }}
  </div>
}

let default = make
