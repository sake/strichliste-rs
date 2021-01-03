type action = Balance(ValueUtils.balance)

type state = Service.user

let reducer = (state, action) => {
  switch action {
    | Balance(balance) => {
      // transmit balance
      state
    }
  }
}


@react.component
let make = (~user: Service.user) => {
  let settings = React.useContext(Settings.settingsCtx)
  let (state, _dispatch) = React.useReducer(reducer, user)

  // user values
  let userName = React.string(state.name)
  let userEmail = email => React.string(email)
  let userBalance = React.string(ValueUtils.formatBalance(settings, state.balance))

  <div>
    <p> {userName} </p>
    {switch state.email {
    | Some(email) => <p> {userEmail(email)} </p>
    | None => React.null
    }}
    <p> {userBalance} </p>
    <button onClick={_ => Service.balanceUser(Belt.Int.toString(state.id), 100)}>{React.string("Click me")}</button>
  </div>
}

let default = make
