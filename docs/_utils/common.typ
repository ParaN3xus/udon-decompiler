#import "/docs/book.typ": cross-link, heading-reference


#let udon-decompiler = link("https://github.com/ParaN3xus/udon-decompiler")[Udon Decompiler]
#let udon = link("https://creators.vrchat.com/worlds/udon/")[Udon]
#let uv = link("https://docs.astral.sh/uv/")[uv]
#let vrc-sdk-license = link("https://hello.vrchat.com/legal/sdk")[VRChat SDK 许可证]
#let vcc = link("https://vcc.docs.vrchat.com/")[VCC]
#let asset-ripper = link("https://github.com/AssetRipper/AssetRipper")[Asset Ripper]
#let numba-scfg = link("https://github.com/numba/numba-scfg")[numba-scfg]

#let udc-issue(no) = link(
  "https://github.com/paran3xus/udon-decompiler/issues/" + str(no),
  "paran3xus/udon-decompiler#" + str(no),
)

#let cross-link-heading(path, heading, body) = cross-link(path, reference: heading-reference(heading), body)
