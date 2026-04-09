#let waybill_truck(
  driver: "", driving_license: "", tractor_license: "", snils: "", 
  vehicle: "", license_plate: "", sts: "", 
  work: "", date: ""
) = {
  set page(paper: "a4", flipped: true, margin: 0.5cm)
  set text(font: "DejaVu Sans", size: 7.2pt)
  
  let field(body, w: auto) = box(width: w, stroke: (bottom: 0.5pt), inset: (bottom: 1pt), outset: (bottom: 1pt))[#body]
  let empty_f(w) = box(width: w, stroke: (bottom: 0.5pt))
  let sub(body) = text(size: 5pt)[#body]
  let h_cell(h, body) = box(height: h, width: 100%, align(center + horizon)[#body])

  [
    #rect(width: 100%, height: 100%, stroke: 0.5pt, inset: 10pt)[
      #grid(
        columns: (1fr, 2fr, 1.2fr),
        align: (left, center, right),
        [
          #v(15pt)
          #box(width: 3.5cm, height: 1.2cm, stroke: 0.5pt)[
            #align(center + horizon)[#text(size: 6pt)[Место для штампа\ организации]]
          ]
        ],
        [
          #text(13pt, weight: "bold")[ПУТЕВОЙ ЛИСТ] #linebreak()
          #text(11pt, weight: "bold")[грузового автомобиля] № #empty_f(2.5cm) #linebreak()
          #v(3pt)
          #empty_f(1cm) #field(date) 20 #empty_f(0.5cm) г.
        ],
        [
          #text(size: 6pt)[
            Типовая межотраслевая форма № 4-С\
            Утверждена постановлением Госкомстата России\
            от 28.11.97 № 78
          ]
          #v(3pt)
          #align(right)[
            #grid(
              columns: (2.5cm, 1.8cm),
              stroke: 0.5pt,
              inset: 3pt,
              align: center,
              [], [*Коды*],
              [Форма по ОКУД], [0345004],
              [по ОКПО], []
            )
          ]
        ]
      )

      #v(5pt)

      #grid(
        columns: (15.8cm, 1fr),
        column-gutter: 10pt,
        [
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            row-gutter: 4pt,
            align: bottom,
            [Организация], [#field(" ООО 'АгроТранс', ИНН 1234567890, г. Москва ", w: 100%)\ #align(center)[#sub[наименование, адрес, номер телефона]]],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr, auto, 2.2cm, auto, 1.5cm),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Марка автомобиля], [#field(vehicle, w: 100%)], [Гос. номер], [#field(license_plate, w: 100%)], [Гаражный №], [#empty_f(100%)],
            [Водитель], [#field(driver, w: 100%)], [Табельный №], [#empty_f(100%)], [], [],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1.5fr, auto, 1fr, auto, 1fr),
            column-gutter: 5pt,
            align: bottom,
            row-gutter: 8pt,
            [Удостоверение №], [#field(driving_license, w: 100%)], [Класс], [#empty_f(100%)], [СНИЛС], [#field(snils, w: 100%)],
            [Лиценз. карточка], [#empty_f(100%)], [Рег. №], [#empty_f(100%)], [Серия], [#empty_f(100%)],
            [Прицеп 1], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [Гаражный №], [#empty_f(100%)],
            [Прицеп 2], [#empty_f(100%)], [Гос. номер], [#empty_f(100%)], [Гаражный №], [#empty_f(100%)],
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            align: bottom,
            [Сопровождающие лица], [#empty_f(100%)]
          )
        ],
        [
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            [
              #grid(
                columns: (2.2cm, 0.8cm),
                stroke: 0.5pt,
                inset: 3pt,
                align: center,
                [], [*Код*],
                [Режим работы], [],
                [Колонна], [],
                [Бригада], []
              )
            ],
            [
              #align(right)[
                #grid(
                  columns: (1.8cm, 0.7cm, 0.7cm, 0.7cm, 0.7cm, 1.2cm, 1.6cm, 1.8cm),
                  stroke: 0.5pt,
                  inset: 2pt,
                  align: center + horizon,
                  grid.cell(rowspan: 2)[*Работа водителя и ТС*],
                  grid.cell(colspan: 4)[время по графику],
                  grid.cell(rowspan: 2)[нулевой\ проб.км],
                  grid.cell(rowspan: 2)[спидом.\ км],
                  grid.cell(rowspan: 2)[время факт.\ ч, мин.]
                  ,
                  [чис.], [мес.], [ч], [мин],
                  [1], [2], [3], [4], [5], [6], [7], [8],
                  [#h_cell(15pt, "выезд")], [], [], [], [], [], [], [],
                  [#h_cell(15pt, "возврат")], [], [], [], [], [], [], []
                )
              ]
            ]
          )
          #v(4pt)
          #align(right)[
            #grid(
              columns: (0.9cm, 0.8cm, 1.1cm, 1.1cm, 1.1cm, 1cm, 1cm, 1cm, 1.2cm),
              stroke: 0.5pt,
              inset: 2pt,
              align: center + horizon,
              grid.cell(colspan: 9)[*Движение горючего*],
              grid.cell(colspan: 2)[горючее],
              grid.cell(rowspan: 2)[выдано],
              grid.cell(colspan: 2)[остаток при],
              grid.cell(rowspan: 2)[сдано],
              grid.cell(rowspan: 2)[коэфф.],
              grid.cell(colspan: 2)[Время работы],
              [марка], [код], [выезде], [возвр.], [спец.], [двиг.],
              [9], [10], [11], [12], [13], [14], [15], [16], [17],
              [#h_cell(12pt, "дизель")], [], [], [], [], [], [], [], [],
              grid.cell(colspan: 2)[#h_cell(12pt, "подпись")], [запр.], [мех.], [мех.], [запр.], [дисп.], [], []
            )
          ]
        ]
      )

      #v(5pt)
      #align(center)[*ЗАДАНИЕ ВОДИТЕЛЮ*]
      #grid(
        columns: (1fr, 1.5cm, 2cm, 2cm, 2.5cm, 1.2cm, 1.2cm, 1.2cm),
        stroke: 0.5pt,
        inset: 3pt,
        align: center + horizon,
        grid.cell(rowspan: 2)[В чье распоряжение\ (наименование и адрес)],
        grid.cell(rowspan: 2)[время\ приб.],
        grid.cell(colspan: 2)[адрес пункта],
        grid.cell(rowspan: 2)[наименование груза],
        grid.cell(rowspan: 2)[ездок],
        grid.cell(rowspan: 2)[расст.\ км],
        grid.cell(rowspan: 2)[тонн],
        [погрузки], [разгрузки],
        [18], [19], [20], [21], [22], [23], [24], [25],
        [#h_cell(18pt, "ООО 'Заказчик'")], [], [Склад], [Поле], [#work], [], [], [],
        [#h_cell(18pt, "")], [], [], [], [], [], [], [],
        [#h_cell(18pt, "")], [], [], [], [], [], [], [],
        grid.cell(colspan: 7, align: right)[*Итого*], []
      )

      #v(6pt)
      #let sig(w) = box(width: w)[
        #empty_f(100%)\
        #v(-5pt)
        #align(center)[#sub[подпись] #h(1cm) #sub[расшифровка подписи]]
      ]
      #let sig_doc(w) = box(width: w)[
        #empty_f(100%)\
        #v(-5pt)
        #align(center)[#sub[должность] #h(0.5cm) #sub[подпись] #h(0.5cm) #sub[расшифровка подписи]]
      ]
      #grid(
        columns: (1.5fr, 1.5fr, 1fr),
        column-gutter: 15pt,
        [
          Водительское удостоверение проверил, задание выдал,\
          выдать горючее #empty_f(1.5cm) литров\
          #v(8pt)
          Диспетчер #sig(4cm)\
          #v(4pt)
          Водитель по состоянию здоровья к управлению допущен\
          #sig_doc(6cm)\
          #v(4pt)
          Прошел послерейсовый медицинский осмотр\
          #sig(6cm)
        ],
        [
          Автомобиль технически исправен.\
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            align: top,
            [Выезд разрешен. Механик], [#sig(100%)]
          )
          #v(4pt)
          #grid(
            columns: (auto, 1fr),
            column-gutter: 5pt,
            align: top,
            [Автомобиль принял: Водитель], [#sig(100%)]
          )
          #v(4pt)
          При возвращении автомобиль #empty_f(2.5cm) #sub[исправен/неисправен]\
          #v(8pt)
          Сдал водитель #sig(4cm)\
          Принял механик #sig(4cm)
        ],
        [
          Отметки организации-владельца\
          автотранспорта\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)\
          #v(4pt)
          #empty_f(100%)
        ]
      )
    ]
  ]
}

#let waybill_tractor(
  driver: "", driving_license: "", tractor_license: "", snils: "", 
  vehicle: "", license_plate: "", sts: "", 
  work: "", date: ""
) = {
  set page(paper: "a4", margin: 1.5cm)
  set text(font: "DejaVu Sans", size: 9pt)

  [
    #align(center, text(14pt, weight: "bold")[ПУТЕВОЙ ЛИСТ ТРАКТОРА])
    #v(0.5cm)

    #grid(
      columns: (1fr, 1fr),
      row-gutter: 1cm,
      [
        *Организация:* ООО "АгроТранс" #linebreak()
        Марка, модель: #vehicle #linebreak()
        Госномер: #license_plate #linebreak()
        СТС/ПСМ: #sts #linebreak()
        Моточасы при выезде: ..........
      ],
      [
        *Механизатор:* #driver #linebreak()
        УТМ: #tractor_license #linebreak()
        СНИЛС: #snils #linebreak()
        Дата: #date
      ]
    )
    
    #v(1cm)
    #line(length: 100%, stroke: 0.5pt)
    
    #v(0.5cm)
    #text(weight: "bold")[ЗАДАНИЕ:]
    #grid(
      columns: (2fr, 1fr, 1fr),
      stroke: 0.5pt,
      inset: 5pt,
      [*Вид работы*], [*Норма*], [*Факт*],
      [#work], [], []
    )

    #v(1cm)
    #grid(
      columns: (1fr, 1fr),
      [Подпись агронома: #box(width: 3cm, stroke: (bottom: 0.5pt))],
      [Подпись механизатора: #box(width: 3cm, stroke: (bottom: 0.5pt))]
    )
  ]
}
