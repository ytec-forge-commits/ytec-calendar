"""Koyomadoの配布用PDF操作説明書を生成する。"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER, TA_LEFT
from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import ParagraphStyle, getSampleStyleSheet
from reportlab.lib.units import mm
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.platypus import (
    BaseDocTemplate,
    Frame,
    Image,
    KeepTogether,
    PageBreak,
    PageTemplate,
    Paragraph,
    Spacer,
    Table,
    TableStyle,
)


ROOT = Path(__file__).resolve().parents[1]
ASSET_DIR = ROOT / "docs" / "manual-assets"
VERSION = json.loads((ROOT / "package.json").read_text(encoding="utf-8"))["version"]
RELEASE_DATE = "2026年7月23日"
OFFICIAL_URL = "https://ytec.cloudfree.jp/ytb/koyomado/"

PAGE_W, PAGE_H = A4
MARGIN_X = 17 * mm
MARGIN_TOP = 16 * mm
MARGIN_BOTTOM = 16 * mm
CONTENT_W = PAGE_W - 2 * MARGIN_X

INK = colors.HexColor("#343041")
MUTED = colors.HexColor("#716b7c")
PURPLE = colors.HexColor("#806f9f")
PURPLE_DARK = colors.HexColor("#5f5278")
PURPLE_PALE = colors.HexColor("#f1edf7")
GREEN = colors.HexColor("#78a88f")
GREEN_PALE = colors.HexColor("#eaf3ee")
SKY_PALE = colors.HexColor("#eaf2f7")
SAND_PALE = colors.HexColor("#f7f1e6")
ROSE_PALE = colors.HexColor("#f8ecee")
LINE = colors.HexColor("#ded9e6")
WHITE = colors.white


def register_fonts() -> None:
    regular = Path(r"C:\Windows\Fonts\BIZ-UDGothicR.ttc")
    bold = Path(r"C:\Windows\Fonts\BIZ-UDGothicB.ttc")
    if not regular.exists() or not bold.exists():
        raise FileNotFoundError("BIZ UDPゴシックが見つかりません。Windows標準フォントを確認してください。")
    pdfmetrics.registerFont(TTFont("KoyomadoRegular", str(regular), subfontIndex=0))
    pdfmetrics.registerFont(TTFont("KoyomadoBold", str(bold), subfontIndex=0))


def build_styles() -> dict[str, ParagraphStyle]:
    samples = getSampleStyleSheet()
    base = dict(
        fontName="KoyomadoRegular",
        textColor=INK,
        wordWrap="CJK",
        splitLongWords=True,
    )
    return {
        "cover_kicker": ParagraphStyle(
            "CoverKicker", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=10, leading=14, textColor=PURPLE, alignment=TA_CENTER,
            spaceAfter=3 * mm, wordWrap="CJK",
        ),
        "cover_title": ParagraphStyle(
            "CoverTitle", parent=samples["Title"], fontName="KoyomadoBold",
            fontSize=30, leading=36, textColor=INK, alignment=TA_CENTER,
            spaceAfter=2 * mm, wordWrap="CJK",
        ),
        "cover_subtitle": ParagraphStyle(
            "CoverSubtitle", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=11.5, leading=18, textColor=MUTED, alignment=TA_CENTER,
            spaceAfter=7 * mm, wordWrap="CJK",
        ),
        "page_kicker": ParagraphStyle(
            "PageKicker", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=8, leading=11, textColor=PURPLE, spaceAfter=1.5 * mm,
            wordWrap="CJK",
        ),
        "h1": ParagraphStyle(
            "H1", parent=samples["Heading1"], fontName="KoyomadoBold",
            fontSize=21, leading=27, textColor=INK, spaceAfter=4 * mm,
            wordWrap="CJK",
        ),
        "h2": ParagraphStyle(
            "H2", parent=samples["Heading2"], fontName="KoyomadoBold",
            fontSize=13.5, leading=19, textColor=PURPLE_DARK,
            spaceBefore=3 * mm, spaceAfter=2 * mm, wordWrap="CJK",
        ),
        "h3": ParagraphStyle(
            "H3", parent=samples["Heading3"], fontName="KoyomadoBold",
            fontSize=10.5, leading=15, textColor=INK, spaceAfter=1.2 * mm,
            wordWrap="CJK",
        ),
        "body": ParagraphStyle(
            "Body", parent=samples["BodyText"], fontSize=9.4, leading=15,
            spaceAfter=2.1 * mm, **base,
        ),
        "body_small": ParagraphStyle(
            "BodySmall", parent=samples["BodyText"], fontSize=8.3, leading=13,
            spaceAfter=1.5 * mm, **base,
        ),
        "body_tiny": ParagraphStyle(
            "BodyTiny", parent=samples["BodyText"], fontSize=7.3, leading=10.5,
            spaceAfter=0, **base,
        ),
        "muted": ParagraphStyle(
            "Muted", parent=samples["BodyText"], fontName="KoyomadoRegular",
            fontSize=8.2, leading=13, textColor=MUTED, wordWrap="CJK",
        ),
        "step_no": ParagraphStyle(
            "StepNo", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=16, leading=20, textColor=WHITE, alignment=TA_CENTER,
        ),
        "card_title": ParagraphStyle(
            "CardTitle", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=10, leading=14, textColor=INK, spaceAfter=1 * mm,
            wordWrap="CJK",
        ),
        "card_body": ParagraphStyle(
            "CardBody", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=8.4, leading=13, textColor=INK, wordWrap="CJK",
        ),
        "code": ParagraphStyle(
            "Code", parent=samples["Code"], fontName="KoyomadoRegular",
            fontSize=7.8, leading=12, textColor=PURPLE_DARK, backColor=PURPLE_PALE,
            borderPadding=2 * mm, wordWrap="CJK",
        ),
        "footer": ParagraphStyle(
            "Footer", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=7.2, leading=9, textColor=MUTED, alignment=TA_CENTER,
        ),
        "link": ParagraphStyle(
            "Link", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=8.5, leading=13, textColor=PURPLE_DARK, alignment=TA_CENTER,
            wordWrap="CJK",
        ),
    }


def p(text: str, style: ParagraphStyle) -> Paragraph:
    return Paragraph(text, style)


def page_title(story: list, styles: dict[str, ParagraphStyle], kicker: str, title: str, lead: str | None = None) -> None:
    story.append(p(kicker.upper(), styles["page_kicker"]))
    story.append(p(title, styles["h1"]))
    if lead:
        story.append(p(lead, styles["body"]))


def screenshot(filename: str, width: float = 164 * mm) -> Image:
    path = ASSET_DIR / filename
    if not path.exists():
        raise FileNotFoundError(f"説明書用画像が見つかりません: {path}")
    image = Image(str(path))
    image.drawWidth = width
    image.drawHeight = width * 600 / 806
    image.hAlign = "CENTER"
    return image


def card(title: str, body: str, styles: dict[str, ParagraphStyle], background=PURPLE_PALE) -> Table:
    inner = [p(title, styles["card_title"]), p(body, styles["card_body"])]
    table = Table([[inner]], colWidths=[CONTENT_W], hAlign="LEFT")
    table.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), background),
        ("BOX", (0, 0), (-1, -1), 0.7, LINE),
        ("LEFTPADDING", (0, 0), (-1, -1), 4 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 4 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 3 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3 * mm),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
    ]))
    return table


def two_cards(items: list[tuple[str, str]], styles: dict[str, ParagraphStyle], backgrounds=None) -> Table:
    backgrounds = backgrounds or [WHITE] * len(items)
    cells = []
    for title, body in items:
        cells.append([p(title, styles["card_title"]), p(body, styles["card_body"])])
    cols = len(cells)
    gap = 4 * mm
    col_w = (CONTENT_W - gap * (cols - 1)) / cols
    row = []
    for index, cell in enumerate(cells):
        row.append(cell)
        if index < cols - 1:
            row.append("")
    widths = []
    for index in range(cols):
        widths.append(col_w)
        if index < cols - 1:
            widths.append(gap)
    table = Table([row], colWidths=widths, hAlign="LEFT")
    commands = [
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 3.2 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 3.2 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 3 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 3 * mm),
    ]
    for index, bg in enumerate(backgrounds):
        cell_index = index * 2
        commands.extend([
            ("BACKGROUND", (cell_index, 0), (cell_index, 0), bg),
            ("BOX", (cell_index, 0), (cell_index, 0), 0.7, LINE),
        ])
    for index in range(cols - 1):
        spacer_index = index * 2 + 1
        commands.extend([
            ("LEFTPADDING", (spacer_index, 0), (spacer_index, 0), 0),
            ("RIGHTPADDING", (spacer_index, 0), (spacer_index, 0), 0),
        ])
    table.setStyle(TableStyle(commands))
    return table


def step(number: int, title: str, body: str, styles: dict[str, ParagraphStyle]) -> Table:
    circle = Table([[p(str(number), styles["step_no"])]], colWidths=[12 * mm], rowHeights=[12 * mm])
    circle.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), PURPLE),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (-1, -1), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 0),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
    ]))
    text = [p(title, styles["card_title"]), p(body, styles["card_body"])]
    table = Table([[circle, text]], colWidths=[16 * mm, CONTENT_W - 16 * mm], hAlign="LEFT")
    table.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (-1, -1), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 1.7 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2.2 * mm),
        ("LINEBELOW", (0, 0), (-1, -1), 0.5, LINE),
    ]))
    return table


def bullet(text: str, styles: dict[str, ParagraphStyle]) -> Paragraph:
    style = ParagraphStyle(
        "BulletInline", parent=styles["body"], leftIndent=5 * mm,
        firstLineIndent=-4 * mm, bulletIndent=0, spaceAfter=1.5 * mm,
    )
    return Paragraph(text, style, bulletText="•")


def data_table(rows: list[tuple[str, str]], styles: dict[str, ParagraphStyle], first_col=44 * mm) -> Table:
    body = [[p(a, styles["body_small"]), p(b, styles["body_small"])] for a, b in rows]
    table = Table(body, colWidths=[first_col, CONTENT_W - first_col], repeatRows=0, hAlign="LEFT")
    commands = [
        ("BOX", (0, 0), (-1, -1), 0.7, LINE),
        ("INNERGRID", (0, 0), (-1, -1), 0.4, LINE),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 2.2 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2.2 * mm),
    ]
    for index in range(len(body)):
        commands.append(("BACKGROUND", (0, index), (0, index), PURPLE_PALE if index % 2 == 0 else SKY_PALE))
    table.setStyle(TableStyle(commands))
    return table


def decorate_page(canvas, doc) -> None:
    canvas.saveState()
    if doc.page > 1:
        canvas.setStrokeColor(LINE)
        canvas.setLineWidth(0.5)
        canvas.line(MARGIN_X, 12 * mm, PAGE_W - MARGIN_X, 12 * mm)
        canvas.setFont("KoyomadoRegular", 7.2)
        canvas.setFillColor(MUTED)
        canvas.drawString(MARGIN_X, 7.8 * mm, f"Koyomado 操作説明書  v{VERSION}")
        canvas.drawRightString(PAGE_W - MARGIN_X, 7.8 * mm, f"{doc.page} / 10")
    canvas.restoreState()


def build_story(styles: dict[str, ParagraphStyle]) -> list:
    story: list = []

    # 1: cover
    logo = Image(str(ROOT / "src" / "assets" / "koyomado-logo.png"), 22 * mm, 22 * mm)
    logo.hAlign = "CENTER"
    story.extend([
        Spacer(1, 4 * mm),
        logo,
        Spacer(1, 5 * mm),
        p("WINDOWS PORTABLE CALENDAR", styles["cover_kicker"]),
        p("Koyomado 操作説明書", styles["cover_title"]),
        p("予定を、いつでも目に入る場所へ。<br/>デスクトップにそっと置いて使える、シンプルな月カレンダーです。", styles["cover_subtitle"]),
        screenshot("calendar.png", 156 * mm),
        Spacer(1, 6 * mm),
        two_cards([
            ("対応環境", "Windows 10 / 11（64bit）<br/>インストール不要"),
            ("この説明書", f"Koyomado v{VERSION}<br/>{RELEASE_DATE}版"),
            ("保存方式", "アプリ横のdataフォルダー<br/>外部通信・暗号化なし"),
        ], styles, [PURPLE_PALE, GREEN_PALE, SKY_PALE]),
        Spacer(1, 5 * mm),
        p(f'<link href="{OFFICIAL_URL}" color="#5f5278">公式ページ: {OFFICIAL_URL}</link>', styles["link"]),
        PageBreak(),
    ])

    # 2: quick start
    page_title(story, styles, "GETTING STARTED", "はじめに - 4ステップで使い始める", "Koyomadoはインストーラーを使わないポータブルアプリです。ZIPを展開したフォルダーが、そのままアプリ本体と保存場所になります。")
    story.extend([
        step(1, "ZIPをすべて展開", "ダウンロードしたZIPを右クリックし、Windowsの「すべて展開」を選びます。ZIPの中から直接起動せず、先に展開してください。", styles),
        step(2, "置き場所を決める", "展開したKoyomadoフォルダーを、ドキュメント、USBメモリ、Google Driveなど今後使う場所へ移します。自動起動をONにした後の移動は避けてください。", styles),
        step(3, "koyomado.exeを起動", "初回起動時、Windowsの警告が出る場合があります。公式ページから入手したファイルであることとSHA-256を確認し、不安がある場合は実行しないでください。", styles),
        step(4, "位置と起動方法を整える", "画面を好きな位置とサイズに調整します。右上の歯車から、必要な場合だけ「Windows起動時に自動起動」をONにします。", styles),
        Spacer(1, 4 * mm),
        card("最初に覚えること", "右上の×で閉じても完全終了せず、画面が隠れてタスクトレイに残ります。再表示はトレイのKoyomadoアイコンを左クリック。完全終了はアイコンを右クリックして「終了」です。", styles, GREEN_PALE),
        Spacer(1, 4 * mm),
        p("Windowsの警告について", styles["h2"]),
        p("現在の配布ファイルにはコード署名がありません。SmartScreenなどの警告は、危険と確定したという意味ではなく、発行元を署名で確認できない場合にも表示されます。公式ページ掲載のSHA-256とダウンロードしたZIPの値を照合できます。", styles["body"]),
        p("PowerShellで確認する場合", styles["h3"]),
        p(f"Get-FileHash .\\koyomado-v{VERSION}-windows-portable.zip -Algorithm SHA256", styles["code"]),
        PageBreak(),
    ])

    # 3: screen overview
    page_title(story, styles, "SCREEN", "画面の見かた", "中央が月カレンダー、左が今日と直近7日間の予定を示すサイドバーです。")
    story.extend([
        screenshot("calendar.png", 155 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("1  月を移動", "上部の左右矢印で前月・翌月へ移動。「今日」は、押した時点の現在日へ戻ります。"),
            ("2  予定を追加", "予定がない日付を左クリックするか、右上・日付内・左側の追加ボタンから登録できます。"),
        ], styles, [PURPLE_PALE, GREEN_PALE]),
        Spacer(1, 3 * mm),
        two_cards([
            ("3  日の予定を確認", "予定のある日付を左クリックすると、その日の予定一覧がポップアップ表示されます。"),
            ("4  表示を整える", "左上付近のボタンでサイドバーを開閉。歯車で背景テーマと自動起動を設定します。"),
        ], styles, [SKY_PALE, SAND_PALE]),
        Spacer(1, 3 * mm),
        p("土曜は青系、日曜と祝日は赤系で表示します。日本の祝日は名前も日付内に表示されます。祝日データはオフラインで、内蔵範囲は1970年から2050年です。", styles["muted"]),
        PageBreak(),
    ])

    # 4: create/edit/delete
    page_title(story, styles, "SCHEDULE", "予定を追加・編集・削除する", "予定名だけでも登録できます。必要に応じて時刻、場所、メモ、色を加えてください。")
    image = screenshot("editor.png", 92 * mm)
    details = [
        p("入力できる内容", styles["h2"]),
        bullet("<b>予定名</b>: 必須、80文字まで", styles),
        bullet("<b>日付・終日</b>: 終日をOFFにすると開始・終了時刻を指定", styles),
        bullet("<b>毎年繰り返す</b>: 誕生日や記念日向け", styles),
        bullet("<b>場所</b>: 任意、100文字まで", styles),
        bullet("<b>メモ</b>: 任意、1000文字まで", styles),
        bullet("<b>予定の色</b>: 6色から選択", styles),
        Spacer(1, 2 * mm),
        p("保存前に下部のプレビューで表示を確認できます。時刻付き予定は、終了時刻を開始時刻より後に設定してください。", styles["body_small"]),
    ]
    side = Table([[image, details]], colWidths=[96 * mm, CONTENT_W - 96 * mm], hAlign="LEFT")
    side.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (0, 0), 4 * mm),
        ("RIGHTPADDING", (1, 0), (1, 0), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 0),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
    ]))
    story.extend([
        side,
        Spacer(1, 4 * mm),
        two_cards([
            ("編集する", "カレンダー上の予定を左クリックするか、日の予定一覧から選ぶと編集画面が開きます。「変更を保存」で反映します。"),
            ("削除する", "編集画面左下、または予定の右クリックメニューから「削除」を選び、確認画面で確定します。削除済み記録は保存データ内に残ります。"),
        ], styles, [SKY_PALE, ROSE_PALE]),
        Spacer(1, 3 * mm),
        card("同じ日に予定が多い場合", "月表示に収まらない予定は「ほか○件」とまとめて表示します。日付を選ぶと一覧ですべて確認でき、5件以上登録した場合もここから編集できます。", styles, GREEN_PALE),
        PageBreak(),
    ])

    # 5: copy and drag
    page_title(story, styles, "COPY AND MOVE", "予定をコピー・移動する", "繰り返し入力する内容は右クリック、日付だけ変えたいときはドラッグ操作が便利です。")
    story.extend([
        two_cards([
            ("右クリックでコピー", "1. 予定を右クリック<br/>2. 「内容をコピー」<br/>3. 貼り付け先の日付を右クリック<br/>4. 「ここに貼り付け」"),
            ("ドラッグで移動", "予定をつかみ、別の日へドラッグして離します。元の日から予定が移動します。"),
            ("Ctrl + ドラッグでコピー", "Ctrlキーを押したまま予定を別の日へドラッグ。元を残し、移動先へ複製します。"),
        ], styles, [PURPLE_PALE, SKY_PALE, GREEN_PALE]),
        Spacer(1, 5 * mm),
        screenshot("calendar.png", 145 * mm),
        Spacer(1, 4 * mm),
        card("コピーされる内容", "予定名、終日／時刻、場所、メモ、色、「毎年繰り返す」の設定をコピーします。貼り付け先の日付だけが新しい日付になります。", styles, SAND_PALE),
        Spacer(1, 3 * mm),
        p("操作の使い分け", styles["h2"]),
        data_table([
            ("同じ内容を複数日に登録", "予定を右クリックしてコピーし、任意の日付を右クリックして貼り付け"),
            ("予定日を変更", "通常のドラッグ"),
            ("元を残して別日にも登録", "Ctrlキーを押したままドラッグ"),
            ("一部を直してから登録", "予定をコピー後、「予定を追加」画面で「内容を貼り付け」して編集"),
        ], styles),
        PageBreak(),
    ])

    # 6: agenda and anniversary
    page_title(story, styles, "AGENDA AND ANNIVERSARY", "日の予定一覧と毎年の記念日", "同じ日に複数の予定がある場合も、日付を選べば一覧で落ち着いて確認できます。")
    left = screenshot("agenda.png", 77 * mm)
    right = screenshot("anniversary.png", 77 * mm)
    images = Table([[left, right]], colWidths=[CONTENT_W / 2, CONTENT_W / 2])
    images.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("ALIGN", (0, 0), (-1, -1), "CENTER"),
        ("LEFTPADDING", (0, 0), (-1, -1), 1 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 1 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 0),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
    ]))
    story.extend([
        images,
        Spacer(1, 3 * mm),
        two_cards([
            ("日の予定一覧", "予定がある日付を左クリックすると、その日の予定を時刻順で表示します。予定を選ぶと編集でき、「この日に予定を追加」から追加入力もできます。"),
            ("毎年繰り返す", "予定の追加・編集画面でONにすると、登録した月日へ毎年表示します。誕生日、創立日、更新日などに使えます。"),
        ], styles, [SKY_PALE, PURPLE_PALE]),
        Spacer(1, 3 * mm),
        p("記念日を編集・削除するとき", styles["h2"]),
        bullet("表示されている年度の記念日を選ぶと、元の1件を編集します。タイトル、月日、色などの変更はすべての年度表示へ反映されます。", styles),
        bullet("記念日を削除すると、その年度だけでなく、過去・未来を含むすべての年度のカレンダー表示から消えます。", styles),
        bullet("誤って削除した記念日を画面上から元に戻す機能はありません。必要な場合は、削除前のdataフォルダーのバックアップから復旧してください。", styles),
        card("2月29日の記念日", "うるう年にだけ2月29日へ表示されます。うるう年以外の2月28日や3月1日へ自動移動はしません。", styles, SAND_PALE),
        PageBreak(),
    ])

    # 7: appearance
    page_title(story, styles, "APPEARANCE", "背景・サイドバー・ウィンドウ", "デスクトップに馴染む8つの背景と、置き方に合わせた2段階の最小幅を用意しています。")
    story.extend([
        screenshot("themes.png", 145 * mm),
        Spacer(1, 3 * mm),
        p("8つの背景テーマ", styles["h2"]),
        p("朝もや / 森の息吹 / 藤の夕暮れ / 陽だまり / 月夜の水面 / 空のそよ風 / 桜かすみ / 白樺の朝", styles["body"]),
        two_cards([
            ("サイドバー表示中", "今日と直近7日間、背景テーマのショートカットを表示。最小幅は806pxです。"),
            ("サイドバー非表示", "カレンダーをコンパクトに表示。最小幅は375pxです。開き直すときは必要な幅まで自動で広がります。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 3 * mm),
        card("位置とサイズを記憶", "移動・サイズ変更・サイドバーの開閉状態を自動保存し、次回は前回の状態で表示します。最小高さは600pxです。画面外になった位置情報は使わず、見える位置へ戻します。", styles, PURPLE_PALE),
        PageBreak(),
    ])

    # 8: tray and autostart
    page_title(story, styles, "TRAY AND STARTUP", "タスクトレイと自動起動", "Koyomadoはデスクトップウィジェットとして使いやすいよう、起動中もタスクバーへは表示しません。")
    flow = Table([[
        [p("×で閉じる", styles["card_title"]), p("画面だけを隠す", styles["card_body"])],
        p("→", styles["h2"]),
        [p("タスクトレイ", styles["card_title"]), p("アプリは動作中", styles["card_body"])],
        p("→", styles["h2"]),
        [p("左クリック", styles["card_title"]), p("画面を再表示", styles["card_body"])],
    ]], colWidths=[48 * mm, 10 * mm, 48 * mm, 10 * mm, 48 * mm])
    flow.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (0, 0), PURPLE_PALE),
        ("BACKGROUND", (2, 0), (2, 0), GREEN_PALE),
        ("BACKGROUND", (4, 0), (4, 0), SKY_PALE),
        ("BOX", (0, 0), (0, 0), 0.7, LINE),
        ("BOX", (2, 0), (2, 0), 0.7, LINE),
        ("BOX", (4, 0), (4, 0), 0.7, LINE),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("ALIGN", (1, 0), (1, 0), "CENTER"),
        ("ALIGN", (3, 0), (3, 0), "CENTER"),
        ("LEFTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 4 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 4 * mm),
    ]))
    story.extend([
        Spacer(1, 3 * mm),
        flow,
        Spacer(1, 6 * mm),
        p("タスクトレイの操作", styles["h2"]),
        data_table([
            ("アイコンを左クリック", "Koyomadoの画面を表示し、手前へ移動"),
            ("アイコンを右クリック", "メニューを表示"),
            ("カレンダーを表示", "隠れている画面を表示"),
            ("終了", "保存後にアプリを完全終了"),
        ], styles),
        Spacer(1, 5 * mm),
        p("Windows起動時に自動起動", styles["h2"]),
        step(1, "右上の歯車を開く", "「表示と起動の設定」を開きます。", styles),
        step(2, "自動起動をON", "「Windows起動時に自動起動」のスイッチを選びます。次回のWindowsサインイン時から起動します。", styles),
        step(3, "フォルダーを移動するときは登録し直す", "移動前に自動起動をOFFにし、移動後のkoyomado.exeから再びONにします。", styles),
        card("起動したのに見えないとき", "タスクトレイのKoyomadoアイコンを左クリックしてください。隠れているアイコンは、通知領域の「隠れているインジケーターを表示します」内にある場合があります。", styles, SAND_PALE),
        PageBreak(),
    ])

    # 9: data and update
    page_title(story, styles, "DATA AND UPDATE", "データ保存・持ち運び・更新", "予定と設定は暗号化せず、koyomado.exeと同じ場所のdataフォルダーへ保存します。")
    story.extend([
        data_table([
            ("calendar-data.json", "予定、削除済み予定、背景テーマ、サイドバー状態"),
            ("calendar-data.backup.json", "予定データを更新する直前のバックアップ"),
            ("window-state.json", "ウィンドウの位置とサイズ"),
            ("window-state.backup.json", "位置情報を更新する直前のバックアップ"),
        ], styles, 58 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("USBメモリ", "Koyomadoフォルダー全体をコピーします。取り外す前にタスクトレイから終了してください。"),
            ("Google Drive", "同期完了後に起動し、同じフォルダーを複数PCから同時に開かないでください。競合の自動解決は行いません。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        p("新しい版へ更新する", styles["h2"]),
        step(1, "Koyomadoを終了", "タスクトレイのアイコンを右クリックし「終了」を選びます。", styles),
        step(2, "dataをバックアップ", "現在のKoyomadoフォルダー内のdataフォルダーを、別の安全な場所へコピーします。", styles),
        step(3, "新しいZIPを展開", "新しいフォルダーへ「すべて展開」します。", styles),
        step(4, "dataを引き継ぐ", "古いKoyomadoフォルダーのdataフォルダーを、新しいKoyomadoフォルダーへコピーします。", styles),
        step(5, "起動して確認", "koyomado.exeを起動し、予定・背景・位置を確認します。自動起動を使っていた場合は、新しい場所からONにし直します。", styles),
        Spacer(1, 3 * mm),
        card("大切な注意", "dataフォルダーを削除したり、新しい空のdataだけを残したりすると、予定を引き継げません。アプリの更新前には必ずフォルダーごとバックアップしてください。", styles, ROSE_PALE),
        PageBreak(),
    ])

    # 10: trouble and reference
    page_title(story, styles, "HELP", "困ったとき・早見表", "画面に予定が見えない場合は、まず月、日付、タスクトレイ、dataフォルダーの順に確認してください。")
    story.extend([
        data_table([
            ("画面が見つからない", "タスクトレイのKoyomadoアイコンを左クリック。隠れているアイコンも確認します。"),
            ("予定が見えない", "表示中の年月を確認し、予定のある日付をクリックして日の一覧を開きます。"),
            ("記念日が翌年に出ない", "編集画面で「毎年繰り返す」がONか確認。2月29日はうるう年だけ表示されます。"),
            ("貼り付けが選べない", "先に予定を右クリックして「内容をコピー」を選びます。"),
            ("起動時の位置がおかしい", "画面外と判断した場合は自動で見える位置へ戻ります。いったん移動して終了し直してください。"),
            ("データが壊れた", "バックアップから自動復旧を試み、壊れたファイルはcorrupt付きの名前で退避します。直らない場合はdataのバックアップを戻します。"),
        ], styles, 46 * mm),
        Spacer(1, 4 * mm),
        p("操作早見表", styles["h2"]),
        two_cards([
            ("左クリック", "予定: 編集<br/>空の日付: 予定を追加<br/>予定がある日付: 日の一覧<br/>トレイアイコン: 再表示"),
            ("右クリック", "予定: コピー・編集・削除<br/>日付: 貼り付け・追加<br/>トレイ: 表示・終了"),
            ("ドラッグ", "通常: 予定を移動<br/>Ctrlを押しながら: 予定をコピー"),
        ], styles, [PURPLE_PALE, GREEN_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        p("仕様上の範囲", styles["h2"]),
        p("Koyomadoには、印刷・予定のPDF出力・外部API・クラウド同期・認証・アクセス解析はありません。予定データは暗号化されないため、機密情報やパスワードの保存には使用しないでください。祝日は1970年から2050年までの内蔵データを使います。", styles["body"]),
        card("お問い合わせ前に用意すると役立つ情報", "Koyomadoのバージョン、Windowsのバージョン、発生した操作、表示されたメッセージ、再現手順。予定の本文や個人情報は送らないでください。", styles, SAND_PALE),
        Spacer(1, 5 * mm),
        p(f'<b>公式ページ</b><br/><link href="{OFFICIAL_URL}" color="#5f5278">{OFFICIAL_URL}</link><br/><br/>利用条件はZIPに同梱したLICENSE.txtをご確認ください。', styles["link"]),
    ])
    return story


def generate(output: Path) -> None:
    register_fonts()
    styles = build_styles()
    output.parent.mkdir(parents=True, exist_ok=True)
    doc = BaseDocTemplate(
        str(output),
        pagesize=A4,
        leftMargin=MARGIN_X,
        rightMargin=MARGIN_X,
        topMargin=MARGIN_TOP,
        bottomMargin=MARGIN_BOTTOM,
        title=f"Koyomado 操作説明書 v{VERSION}",
        author="Y-TEC",
        subject="Koyomado Windowsポータブルカレンダーの操作説明書",
        creator="Koyomado manual generator",
    )
    frame = Frame(
        doc.leftMargin,
        doc.bottomMargin,
        doc.width,
        doc.height,
        id="content",
        leftPadding=0,
        rightPadding=0,
        topPadding=0,
        bottomPadding=0,
    )
    doc.addPageTemplates([PageTemplate(id="main", frames=[frame], onPage=decorate_page)])
    doc.build(build_story(styles))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        default=ROOT / "output" / "pdf" / "Koyomado操作説明書.pdf",
        help="出力するPDFファイル",
    )
    args = parser.parse_args()
    generate(args.output.resolve())
    print(f"操作説明書を作成しました: {args.output.resolve()}")


if __name__ == "__main__":
    main()
