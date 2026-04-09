#let waybill(driver: "", vehicle: "", work: "", date: "") = {
  set page(paper: "a4", margin: 2cm)
  set text(font: "DejaVu Sans", size: 12pt)

  align(center, text(20pt, weight: "bold")[ПУТЕВОЙ ЛИСТ])
  
  v(1cm)
  
  grid(
    columns: (1fr, 1fr),
    row-gutter: 1.5cm,
    [#text(weight: "bold")[Дата:] #date],
    [#text(weight: "bold")[Номер документа:] № 001],
    
    [#text(weight: "bold")[Водитель:]],
    [#driver],
    
    [#text(weight: "bold")[Транспортное средство:]],
    [#vehicle],
    
    [#text(weight: "bold")[Вид работ:]],
    [#work],
  )
  
  v(2cm)
  
  line(length: 100%, stroke: 0.5pt)
  v(0.5cm)
  grid(
    columns: (1fr, 1fr),
    [Подпись диспетчера: #box(width: 3cm, stroke: (bottom: 0.5pt))],
    [Подпись водителя: #box(width: 3cm, stroke: (bottom: 0.5pt))],
  )
}

#waybill(
  driver: "{{driver}}",
  vehicle: "{{vehicle}}",
  work: "{{work}}",
  date: "{{date}}"
)
