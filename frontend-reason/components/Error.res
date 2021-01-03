@react.component
let make = (~error: FetchUtils.fetchError) => {
  Js.log(error)
  <h1> {React.string("fucked up!")} </h1>
}

let default = make
