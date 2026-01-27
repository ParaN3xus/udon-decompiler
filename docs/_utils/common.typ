#import "@preview/cheq:0.3.0": checklist


#let udon-decompiler = link("https://github.com/ParaN3xus/udon-decompiler")[Udon Decompiler]
#let udon = link("https://creators.vrchat.com/worlds/udon/")[Udon]
#let uv = link("https://docs.astral.sh/uv/")[uv]
#let vrc-sdk-license = link("https://hello.vrchat.com/legal/sdk")[VRChat SDK 许可证]
#let vcc = link("https://vcc.docs.vrchat.com/")[VCC]
#let asset-ripper = link("https://github.com/AssetRipper/AssetRipper")[Asset Ripper]


#let custom-rules(body) = {
  show: checklist
  // set heading(numbering: "1.")
  body
}
