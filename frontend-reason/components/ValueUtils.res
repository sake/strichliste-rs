@decco
type balance = int

type numberFormat
type numberFormatOptions = {
  style: string,
  currency: string,
  signDisplay: string,
}

@bs.new @bs.scope("Intl")
external createNf: (string, numberFormatOptions) => numberFormat = "NumberFormat"
@bs.send
external formatNf: (numberFormat, float) => string = "format"

let formatBalance = (s: Settings.settings, b: balance) => {
  let bf = Belt.Float.fromInt(b) /. 100.0
  let nf = createNf(
    s.i18n.language,
    {style: "currency", currency: s.i18n.currency.alpha3, signDisplay: "always"},
  )

  nf->formatNf(bf)
}
