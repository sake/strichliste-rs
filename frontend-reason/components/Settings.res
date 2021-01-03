@decco.decode
type common = {idleTimeout: int}

@decco.decode
type currency = {alpha3: string}

@decco.decode
type i18n = {currency: currency, language: string}

@decco.decode
type settings = {
  common: common,
  // article: article,
  // paypal: paypal,
  // user: user,
  i18n: i18n,
  // account: account,
  // payment: payment,
}

@decco.decode
type settingsWrapper = {settings: settings}

let fetchSettings = callback => {
  let cbWrapper = data => {
    switch data {
    | FetchUtils.Fetched(v) => callback(v)
    | _ => ()
    }
  }
  FetchUtils.fetchTemplate(
    Consts.settingsApi(),
    settingsWrapper_decode,
    sw => sw.settings,
    cbWrapper,
  )
}

let defaultSettings: settings = {
  common: {
    idleTimeout: 60000,
  },
  i18n: {
    language: "en",
    currency: {
      alpha3: "EUR",
    },
  },
}

let settingsCtx = React.createContext(defaultSettings)
let make = React.Context.provider(settingsCtx)
