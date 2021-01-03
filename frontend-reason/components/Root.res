@react.component
let make = (~children) => {
  <>
    <Next.Head>
      <title key="title">{ReasonReact.string("Strichliste")}</title>
    </Next.Head>
    
    <div className="md:container md:mx-auto"> <Nav /> children </div>
  </>
}

let default = make
