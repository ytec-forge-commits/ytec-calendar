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
RELEASE_DATE = "2026年8月23日"
OFFICIAL_URL = "https://ytec.cloudfree.jp/ytb/koyomado/"
SOURCE_URL = "https://github.com/ytec-forge-commits/ytec-calendar"
GOOGLE_CONSOLE_URL = "https://console.cloud.google.com/"
GOOGLE_PROJECT_CREATE_URL = "https://console.cloud.google.com/projectcreate"
GOOGLE_CALENDAR_API_URL = "https://console.cloud.google.com/apis/library/calendar-json.googleapis.com"
GOOGLE_AUTH_OVERVIEW_URL = "https://console.cloud.google.com/auth/overview"
GOOGLE_AUTH_AUDIENCE_URL = "https://console.cloud.google.com/auth/audience"
GOOGLE_AUTH_CLIENTS_URL = "https://console.cloud.google.com/auth/clients"
GOOGLE_CREDENTIALS_URL = "https://developers.google.com/workspace/guides/create-credentials#desktop-app"
GOOGLE_USER_DATA_URL = "https://developers.google.com/terms/api-services-user-data-policy"
GOOGLE_AUDIENCE_HELP_URL = "https://support.google.com/cloud/answer/15549945?hl=ja"
GOOGLE_VERIFICATION_HELP_URL = "https://support.google.com/cloud/answer/13464323?hl=ja"
TOTAL_PAGES = 27

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
    regular = ROOT / "src" / "assets" / "fonts" / "LINESeedJP-Regular.ttf"
    bold = ROOT / "src" / "assets" / "fonts" / "LINESeedJP-Bold.ttf"
    if not regular.exists() or not bold.exists():
        raise FileNotFoundError("同梱したLINE Seed JPフォントが見つかりません。")
    pdfmetrics.registerFont(TTFont("KoyomadoRegular", str(regular)))
    pdfmetrics.registerFont(TTFont("KoyomadoBold", str(bold)))


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
        "ui_header": ParagraphStyle(
            "UiHeader", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=8.2, leading=11, textColor=INK, wordWrap="CJK",
        ),
        "ui_nav": ParagraphStyle(
            "UiNav", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=7.1, leading=10, textColor=MUTED, wordWrap="CJK",
        ),
        "ui_nav_active": ParagraphStyle(
            "UiNavActive", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=7.1, leading=10, textColor=PURPLE_DARK, wordWrap="CJK",
        ),
        "ui_title": ParagraphStyle(
            "UiTitle", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=8, leading=11, textColor=INK, spaceAfter=0.5 * mm,
            wordWrap="CJK",
        ),
        "ui_body": ParagraphStyle(
            "UiBody", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=7, leading=10.5, textColor=MUTED, wordWrap="CJK",
        ),
        "ui_badge": ParagraphStyle(
            "UiBadge", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=10, leading=12, textColor=WHITE, alignment=TA_CENTER,
        ),
        "roadmap_title": ParagraphStyle(
            "RoadmapTitle", parent=samples["Normal"], fontName="KoyomadoBold",
            fontSize=7.5, leading=10.5, textColor=INK, alignment=TA_CENTER,
            spaceAfter=0.4 * mm, wordWrap="CJK",
        ),
        "roadmap_body": ParagraphStyle(
            "RoadmapBody", parent=samples["Normal"], fontName="KoyomadoRegular",
            fontSize=6.4, leading=9, textColor=MUTED, alignment=TA_CENTER,
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
    aspect_ratio = image.imageHeight / image.imageWidth
    image.drawWidth = width
    image.drawHeight = width * aspect_ratio
    image.hAlign = "CENTER"
    return image


def screenshot_figure(
    filename: str,
    caption: str,
    styles: dict[str, ParagraphStyle],
    width: float = 164 * mm,
) -> KeepTogether:
    return KeepTogether([
        screenshot(filename, width),
        Spacer(1, 1.2 * mm),
        p(caption, styles["muted"]),
    ])


def url_link(label: str, url: str, styles: dict[str, ParagraphStyle]) -> Paragraph:
    return p(f'<link href="{url}" color="#5f5278"><b>{label}</b>: {url}</link>', styles["body_small"])


def screenshot_pair(
    left: tuple[str, str],
    right: tuple[str, str],
    styles: dict[str, ParagraphStyle],
) -> Table:
    gap = 4 * mm
    cell_width = (CONTENT_W - gap) / 2
    image_width = cell_width - 5 * mm
    cells = []
    for filename, caption in (left, right):
        cells.append([
            screenshot(filename, image_width),
            Spacer(1, 1.2 * mm),
            p(caption, styles["body_tiny"]),
        ])
    table = Table([[cells[0], "", cells[1]]], colWidths=[cell_width, gap, cell_width], hAlign="LEFT")
    table.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("BACKGROUND", (0, 0), (0, 0), colors.HexColor("#fbfafc")),
        ("BACKGROUND", (2, 0), (2, 0), colors.HexColor("#fbfafc")),
        ("BOX", (0, 0), (0, 0), 0.6, LINE),
        ("BOX", (2, 0), (2, 0), 0.6, LINE),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (-1, -1), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 2 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2 * mm),
    ]))
    return table


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
        firstLineIndent=-4 * mm, spaceAfter=1.5 * mm,
    )
    return Paragraph(f"・{text}", style)


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


def compact_steps(
    items: list[tuple[int, str, str]],
    width: float,
    styles: dict[str, ParagraphStyle],
) -> Table:
    rows = []
    for number, title, body in items:
        badge = Table(
            [[p(str(number), styles["ui_badge"])]],
            colWidths=[8 * mm],
            rowHeights=[8 * mm],
        )
        badge.setStyle(TableStyle([
            ("BACKGROUND", (0, 0), (-1, -1), PURPLE),
            ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
            ("LEFTPADDING", (0, 0), (-1, -1), 0),
            ("RIGHTPADDING", (0, 0), (-1, -1), 0),
            ("TOPPADDING", (0, 0), (-1, -1), 0),
            ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
        ]))
        rows.append([badge, [p(title, styles["ui_title"]), p(body, styles["ui_body"])]] )

    table = Table(rows, colWidths=[11 * mm, width - 11 * mm], hAlign="LEFT")
    table.setStyle(TableStyle([
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (-1, -1), 1.2 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 1.2 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 1.5 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 1.5 * mm),
        ("LINEBELOW", (0, 0), (-1, -2), 0.4, LINE),
    ]))
    return table


def roadmap(items: list[tuple[int, str, str]], styles: dict[str, ParagraphStyle]) -> Table:
    chunks = [chunk for chunk in (items[:4], items[4:]) if chunk]
    rows = []
    for chunk_index, chunk in enumerate(chunks):
        col_width = CONTENT_W / len(chunk)
        cells = []
        for number, title, body in chunk:
            badge = Table(
                [[p(str(number), styles["ui_badge"])]],
                colWidths=[7 * mm],
                rowHeights=[7 * mm],
                hAlign="CENTER",
            )
            badge.setStyle(TableStyle([
                ("BACKGROUND", (0, 0), (-1, -1), PURPLE),
                ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
                ("LEFTPADDING", (0, 0), (-1, -1), 0),
                ("RIGHTPADDING", (0, 0), (-1, -1), 0),
                ("TOPPADDING", (0, 0), (-1, -1), 0),
                ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
            ]))
            cells.append([badge, p(title, styles["roadmap_title"]), p(body, styles["roadmap_body"])])
        row = Table([cells], colWidths=[col_width] * len(chunk), hAlign="LEFT")
        row.setStyle(TableStyle([
            ("BACKGROUND", (0, 0), (-1, -1), PURPLE_PALE if chunk_index == 0 else GREEN_PALE),
            ("BOX", (0, 0), (-1, -1), 0.7, LINE),
            ("INNERGRID", (0, 0), (-1, -1), 0.5, LINE),
            ("VALIGN", (0, 0), (-1, -1), "TOP"),
            ("LEFTPADDING", (0, 0), (-1, -1), 2 * mm),
            ("RIGHTPADDING", (0, 0), (-1, -1), 2 * mm),
            ("TOPPADDING", (0, 0), (-1, -1), 2 * mm),
            ("BOTTOMPADDING", (0, 0), (-1, -1), 2 * mm),
        ]))
        rows.append([row])
        if chunk_index == 0 and len(chunks) > 1:
            rows.append([p("↓  続いてGoogle Auth Platformを設定", styles["muted"])])

    outer = Table(rows, colWidths=[CONTENT_W], hAlign="LEFT")
    outer.setStyle(TableStyle([
        ("ALIGN", (0, 0), (-1, -1), "CENTER"),
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (-1, -1), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 1 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 1 * mm),
    ]))
    return outer


def console_diagram(
    product: str,
    project: str,
    nav_items: list[str],
    active: str,
    heading: str,
    rows: list[tuple[int, str, str]],
    styles: dict[str, ParagraphStyle],
) -> Table:
    top = Table(
        [[
            p(f"Google Cloud  /  {product}", styles["ui_header"]),
            p(f"プロジェクト: {project}", styles["ui_nav_active"]),
            p("検索", styles["ui_nav"]),
        ]],
        colWidths=[57 * mm, 72 * mm, CONTENT_W - 129 * mm],
    )
    top.setStyle(TableStyle([
        ("BACKGROUND", (0, 0), (-1, -1), colors.HexColor("#f7f7fa")),
        ("BOX", (0, 0), (-1, -1), 0.7, LINE),
        ("INNERGRID", (0, 0), (-1, -1), 0.4, LINE),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("LEFTPADDING", (0, 0), (-1, -1), 2.4 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 2.4 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 2.2 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2.2 * mm),
    ]))

    nav_rows = [[p(label, styles["ui_nav_active"] if label == active else styles["ui_nav"])] for label in nav_items]
    nav = Table(nav_rows, colWidths=[38 * mm], hAlign="LEFT")
    nav_commands = [
        ("BACKGROUND", (0, 0), (-1, -1), colors.HexColor("#faf9fc")),
        ("BOX", (0, 0), (-1, -1), 0.7, LINE),
        ("VALIGN", (0, 0), (-1, -1), "MIDDLE"),
        ("LEFTPADDING", (0, 0), (-1, -1), 3 * mm),
        ("RIGHTPADDING", (0, 0), (-1, -1), 2 * mm),
        ("TOPPADDING", (0, 0), (-1, -1), 2 * mm),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 2 * mm),
    ]
    active_index = nav_items.index(active)
    nav_commands.extend([
        ("BACKGROUND", (0, active_index), (0, active_index), PURPLE_PALE),
        ("LINEBEFORE", (0, active_index), (0, active_index), 2, PURPLE),
    ])
    nav.setStyle(TableStyle(nav_commands))

    body_width = CONTENT_W - 38 * mm
    body = [p(heading, styles["ui_header"]), compact_steps(rows, body_width - 8 * mm, styles)]
    lower = Table([[nav, body]], colWidths=[38 * mm, body_width], hAlign="LEFT")
    lower.setStyle(TableStyle([
        ("BOX", (0, 0), (-1, -1), 0.7, LINE),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LEFTPADDING", (0, 0), (0, -1), 0),
        ("RIGHTPADDING", (0, 0), (0, -1), 0),
        ("TOPPADDING", (0, 0), (0, -1), 0),
        ("BOTTOMPADDING", (0, 0), (0, -1), 0),
        ("LEFTPADDING", (1, 0), (1, -1), 4 * mm),
        ("RIGHTPADDING", (1, 0), (1, -1), 4 * mm),
        ("TOPPADDING", (1, 0), (1, -1), 3 * mm),
        ("BOTTOMPADDING", (1, 0), (1, -1), 3 * mm),
    ]))

    screen = Table([[top], [lower]], colWidths=[CONTENT_W], hAlign="LEFT")
    screen.setStyle(TableStyle([
        ("LEFTPADDING", (0, 0), (-1, -1), 0),
        ("RIGHTPADDING", (0, 0), (-1, -1), 0),
        ("TOPPADDING", (0, 0), (-1, -1), 0),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 0),
    ]))
    return screen


def decorate_page(canvas, doc) -> None:
    canvas.saveState()
    if doc.page > 1:
        canvas.setStrokeColor(LINE)
        canvas.setLineWidth(0.5)
        canvas.line(MARGIN_X, 12 * mm, PAGE_W - MARGIN_X, 12 * mm)
        canvas.setFont("KoyomadoRegular", 7.2)
        canvas.setFillColor(MUTED)
        canvas.drawString(MARGIN_X, 7.8 * mm, f"Koyomado 操作説明書  v{VERSION}")
        canvas.setFont("Helvetica", 7.2)
        canvas.drawRightString(PAGE_W - MARGIN_X, 7.8 * mm, f"{doc.page} / {TOTAL_PAGES}")
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
        screenshot("calendar-v1.png", 156 * mm),
        Spacer(1, 6 * mm),
        two_cards([
            ("対応環境", "Windows 10 / 11（64bit）<br/>インストール不要"),
            ("この説明書", f"Koyomado v{VERSION}<br/>{RELEASE_DATE}・署名前ベータ版"),
            ("保存と通信", "予定はアプリ横へ保存<br/>Google連携は任意・初期OFF"),
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
        card("標準はタスクバーだけに表示", "初期設定では通常のWindowsアプリと同じく、最小化するとタスクバーへ残り、右上の×で終了します。歯車から「タスクトレイのみ」または「両方」へ変更できます。", styles, GREEN_PALE),
        Spacer(1, 4 * mm),
        p("Windowsの警告について", styles["h2"]),
        p(f"v{VERSION}はコード署名前のベータ版です。SmartScreenなどの警告は、危険と確定したという意味ではなく、発行元を署名で確認できない場合にも表示されます。公式ページ掲載のSHA-256とダウンロードしたZIPの値を照合し、入手元を確認してください。", styles["body"]),
        p("PowerShellで確認する場合", styles["h3"]),
        p(f"Get-FileHash .\\koyomado-v{VERSION}-windows-portable.zip -Algorithm SHA256", styles["code"]),
        PageBreak(),
    ])

    # 3: screen overview
    page_title(story, styles, "SCREEN", "画面の見かた", "中央が月カレンダー、左が今日と直近7日間の予定を示すサイドバーです。")
    story.extend([
        screenshot("calendar-v1.png", 155 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("1  月を移動", "上部の左右矢印で前月・翌月へ移動。「今日」は、押した時点の現在日へ戻ります。"),
            ("2  予定を追加", "予定がない日付を左クリックするか、右上・日付内・左側の追加ボタンから登録できます。"),
        ], styles, [PURPLE_PALE, GREEN_PALE]),
        Spacer(1, 3 * mm),
        two_cards([
            ("3  日の予定を確認", "予定のある日付を左クリックすると、その日の予定一覧がポップアップ表示されます。"),
            ("4  表示を整える", "上部のボタンでサイドバーを開閉。歯車で背景、表示先、自動起動、Google連携を設定します。"),
        ], styles, [SKY_PALE, SAND_PALE]),
        Spacer(1, 3 * mm),
        p("土曜は青系、日曜と祝日は赤系で表示します。日本の祝日は名前も日付内に表示されます。祝日データはオフラインで、内蔵範囲は1970年から2050年です。", styles["muted"]),
        PageBreak(),
    ])

    # 4: create/edit/delete
    page_title(story, styles, "SCHEDULE", "予定を追加・編集・削除する", "開始日と終了日を持つ予定を登録できます。必要に応じて時刻、場所、メモ、色を加えてください。")
    image = screenshot("period-editor-v1.png", 96 * mm)
    details = [
        p("入力できる内容", styles["h2"]),
        bullet("<b>予定名</b>: 必須、80文字まで", styles),
        bullet("<b>開始日・終了日</b>: 同日または複数日を指定", styles),
        bullet("<b>終日</b>: 休み、出張、記念日など時刻が不要な予定", styles),
        bullet("<b>開始・終了時刻</b>: 終日をOFFにすると表示", styles),
        bullet("<b>繰り返し</b>: 毎日、毎週、毎月、毎年", styles),
        bullet("<b>場所</b>: 任意、100文字まで", styles),
        bullet("<b>メモ</b>: 任意、1000文字まで", styles),
        bullet("<b>予定の色</b>: 6色から選択", styles),
        Spacer(1, 2 * mm),
        p("開始時刻を変えると、終了は1時間後へ自動設定されます。その後に終了日・終了時刻を手動で変更できます。", styles["body_small"]),
    ]
    side = Table([[image, details]], colWidths=[100 * mm, CONTENT_W - 100 * mm], hAlign="LEFT")
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

    # 5: multi-day events
    page_title(story, styles, "MULTI-DAY", "連休・出張・日をまたぐ予定", "9月1日から3日までの休みなどは、1件の期間予定として登録できます。")
    story.extend([
        screenshot("multi-day-v1.png", 152 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("終日の複数日予定", "開始日を9月1日、終了日を9月3日、終日をONにすると、1日・2日・3日の各日に同じ予定を表示します。"),
            ("時刻付きの日またぎ", "終日をOFFにし、開始を9月1日23:30、終了を9月2日0:30のように指定します。2日以上先も選べます。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 3 * mm),
        data_table([
            ("開始時刻を変更", "終了日時を開始の1時間後へ自動設定。23:30なら翌日0:30"),
            ("終了日時を変更", "自動設定後も、終了日と終了時刻を自由に上書き可能"),
            ("カレンダー表示", "期間中の各日に表示。どの日から開いても同じ開始・終了日時を編集"),
            ("直近7日間", "同じ期間予定を日数分重複させず、1件として表示"),
        ], styles, 46 * mm),
        Spacer(1, 3 * mm),
        card("Googleカレンダーとの期間同期", "複数日の終日予定と日をまたぐ時刻付き予定も、開始・終了日時を保って双方向同期します。終日予定の終了日が1日ずれないよう、Google側の排他的終了日をKoyomado内で変換します。", styles, PURPLE_PALE),
        PageBreak(),
    ])

    # 6: copy and drag
    page_title(story, styles, "COPY AND MOVE", "予定をコピー・移動する", "繰り返し入力する内容は右クリック、日付だけ変えたいときはドラッグ操作が便利です。")
    story.extend([
        two_cards([
            ("右クリックでコピー", "1. 予定を右クリック<br/>2. 「内容をコピー」<br/>3. 貼り付け先の日付を右クリック<br/>4. 「ここに貼り付け」"),
            ("ドラッグで移動", "予定をつかみ、別の日へドラッグして離します。元の日から予定が移動します。"),
            ("Ctrl + ドラッグでコピー", "Ctrlキーを押したまま予定を別の日へドラッグ。元を残し、移動先へ複製します。"),
        ], styles, [PURPLE_PALE, SKY_PALE, GREEN_PALE]),
        Spacer(1, 5 * mm),
        screenshot("calendar-v1.png", 145 * mm),
        Spacer(1, 4 * mm),
        card("コピーされる内容", "予定名、期間、終日／時刻、場所、メモ、色、繰り返し条件をコピーします。複数日予定では日数を保ったまま、貼り付け先を新しい開始日にします。Ctrl＋ドラッグした繰り返し予定の1回分は、独立した通常予定としてコピーします。", styles, SAND_PALE),
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

    # 7: recurrence setup
    page_title(story, styles, "RECURRENCE", "繰り返し予定を設定する", "周期、間隔、曜日、終了条件を組み合わせて、定期予定と記念日を登録できます。")
    story.extend([
        screenshot("recurrence-v1.png", 145 * mm),
        Spacer(1, 4 * mm),
        data_table([
            ("毎日", "1日ごと、2日ごとなど。連日の当番や服薬予定"),
            ("毎週", "複数曜日を選択可能。月・水・金、隔週など"),
            ("毎月", "同じ日付、または第何週の同じ曜日"),
            ("毎年", "誕生日、記念日、更新日。2月29日はうるう年だけ表示"),
            ("終了条件", "終了なし、指定日まで、指定回数"),
        ], styles, 37 * mm),
        Spacer(1, 3 * mm),
        card("複数日と繰り返しの組み合わせ", "開始日から終了日までの日数も各回へ引き継ぎます。たとえば毎月1日から3日までの出張を登録すると、毎回3日間の予定として表示します。", styles, GREEN_PALE),
        PageBreak(),
    ])

    # 8: recurrence operations and agenda
    page_title(story, styles, "RECURRENCE SCOPE", "1回だけ・全体の編集と記念日", "繰り返し予定を開くと、今回だけ変えるか、シリーズ全体を変えるかを選べます。")
    story.extend([
        two_cards([
            ("この予定のみ", "選んだ回だけ内容や日付を変更します。削除すると、その回だけを除外します。通常ドラッグも、その回だけを移動します。"),
            ("繰り返し全体", "元の予定名、時刻、期間、周期、色などを更新します。削除すると、過去・未来の表示と個別例外を含むシリーズ全体を削除します。"),
        ], styles, [SKY_PALE, PURPLE_PALE]),
        Spacer(1, 4 * mm),
        screenshot("agenda.png", 118 * mm),
        Spacer(1, 3 * mm),
        p("日の予定一覧", styles["h2"]),
        p("予定がある日付を左クリックすると、その日の予定を一覧表示します。同じ日に5件以上あってもここですべて確認でき、予定を選ぶと編集できます。", styles["body"]),
        p("誕生日・記念日", styles["h2"]),
        bullet("繰り返し周期で「毎年」を選び、登録する月日を開始日にします。", styles),
        bullet("特定の年だけ内容を変える場合は「この予定のみ」、今後も含めて変える場合は「繰り返し全体」を選びます。", styles),
        bullet("記念日全体を削除すると、過去・未来のすべての年度表示から消えます。削除前に必要ならdataフォルダーをバックアップしてください。", styles),
        card("Googleから取り込んだ複雑な繰り返し", "Google独自の繰り返し条件はKoyomadoで表示・同期できます。周期そのものの変更がロックされている場合はGoogleカレンダー側で変更してください。個別回の編集はKoyomadoでも行えます。", styles, SAND_PALE),
        PageBreak(),
    ])

    # 9: appearance
    page_title(story, styles, "APPEARANCE", "背景・サイドバー・ウィンドウ", "デスクトップに馴染む8つの背景と、置き方に合わせた2段階の最小幅を用意しています。")
    story.extend([
        screenshot("settings-v1.png", 126 * mm),
        Spacer(1, 3 * mm),
        p("8つの背景テーマ", styles["h2"]),
        p("朝もや / 森の息吹 / 藤の夕暮れ / 陽だまり / 月夜の水面 / 空のそよ風 / 桜かすみ / 白樺の朝", styles["body"]),
        two_cards([
            ("サイドバー表示中", "今日と直近7日間、背景テーマのショートカットを表示。最小幅は806pxです。"),
            ("サイドバー非表示", "カレンダーをコンパクトに表示。最小幅は375pxです。開き直すときは必要な幅まで自動で広がります。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 3 * mm),
        card("表示倍率をスライダーで調整", "右上の歯車にある「表示サイズ」で80～130%を5%刻みで選べます。変更中にすぐ見え方を確認でき、再起動後も維持します。「100%に戻す」で初期値へ戻せます。", styles, SAND_PALE),
        Spacer(1, 3 * mm),
        card("モニター構成ごとに位置を記憶", "移動・サイズ変更・サイドバーの開閉状態を自動保存します。3画面、2画面など構成ごとに最後の位置を分けて記憶するため、以前の構成へ戻ると、その構成で保存した位置へ戻ります。画面外の位置は使わず、見える位置へ戻します。", styles, PURPLE_PALE),
        PageBreak(),
    ])

    # 10: taskbar, tray and autostart
    page_title(story, styles, "WINDOW DISPLAY", "タスクバー・トレイ・自動起動", "右上の歯車から、普段の使い方に合う表示先を選べます。初期設定はタスクバーのみです。")
    story.extend([
        data_table([
            ("タスクバーのみ（標準）", "最小化するとタスクバーへ残ります。×で閉じるとアプリを終了。トレイアイコンは表示しません。"),
            ("タスクトレイのみ", "最小化または×で画面を隠します。タスクバーには残らず、トレイアイコンから再表示・終了します。"),
            ("両方", "起動中はタスクバーとトレイの両方へ表示。最小化はタスクバー、×では画面を隠してトレイへ残します。"),
        ], styles, 53 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("トレイから再表示", "アイコンを左クリックするか、右クリックして「カレンダーを表示」。完全終了は右クリックの「終了」。"),
            ("表示先を変えた直後", "タスクバー・トレイの表示をすぐに切り替えます。予定とウィンドウ位置は変わりません。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 5 * mm),
        p("Windows起動時に自動起動", styles["h2"]),
        step(1, "右上の歯車を開く", "「表示と起動の設定」を開きます。", styles),
        step(2, "自動起動をON", "「Windows起動時に自動起動」のスイッチを選びます。次回のWindowsサインイン時から起動します。Google Driveなどの準備が遅い場合は、実行ファイルが利用可能になるまで最大5分待機します。", styles),
        step(3, "フォルダーを移動するときは登録し直す", "移動前に自動起動をOFFにし、移動後のkoyomado.exeから再びONにします。", styles),
        card("起動したのに見えないとき", "表示先がトレイを含む場合は、通知領域と「隠れているインジケーター」を確認します。保存位置が現在のモニター構成の画面外なら、Koyomadoは見える位置へ自動的に戻します。", styles, SAND_PALE),
        PageBreak(),
    ])

    # 11: reminder setup and popup
    page_title(story, styles, "REMINDER", "予定の通知を設定する", "予定ごとに開始時刻のどのくらい前に知らせるかを設定します。通知はKoyomadoが起動している間だけ動作します。")
    story.extend([
        step(1, "予定の追加・編集を開く", "予定名、日時などを入力し、画面下部の「通知」までスクロールします。", styles),
        step(2, "通知時間を選ぶ", "よく使う通知時間の10分前・30分前・1時間前・3時間前・6時間前・12時間前・1日前は、クリックだけで複数選択できます。選択中の項目をもう一度押すと解除します。「通知を追加」では数値と「分前」「時間前」「日前」を自由に指定できます。上限は28日前、予定1件につきGoogleのメール通知を含めて最大5件です。", styles),
        step(3, "Googleへ送る方法を確認", "新しくKoyomadoで作る予定は「上の通知時刻をGoogleにも保存する（おすすめ）」が初期選択です。通知を追加・変更・削除した場合も、この選択へ自動的に切り替わります。Googleから取得したメール通知も維持します。", styles),
        step(4, "保存してKoyomadoを起動しておく", "通知時刻になると画面右下に予定名、日時、場所を表示します。トレイへ隠している場合もKoyomadoの画面を表示します。", styles),
        Spacer(1, 4 * mm),
        two_cards([
            ("通知音は設定秒数で停止", "初期値は12秒です。歯車で3～60秒から選べます。先に「OK（音を止める）」を押した場合はすぐ止まり、ポップアップも閉じます。"),
            ("ポップアップは残る", "音が自動停止しても予定のポップアップは残ります。「予定を開く」で編集画面へ、「OK」で確認済みにします。"),
            ("終日予定", "開始日の0:00を基準にします。「Googleカレンダーの既定通知を使う」を選ぶと、上の通知時刻はGoogleへ送られません。Google側の既定が30分前なら前日の23:30です。"),
        ], styles, [GREEN_PALE, SKY_PALE, SAND_PALE]),
        Spacer(1, 4 * mm),
        card("通知を見逃した場合", "スリープ復帰や処理の遅れを考慮し、通知時刻から約2分以内は表示します。それより長くKoyomadoが終了・停止していた間の通知を、後からまとめて鳴らすことはありません。", styles, ROSE_PALE),
        PageBreak(),
    ])

    # 12: notification sounds
    page_title(story, styles, "NOTIFICATION SOUND", "通知音・音量・自分の音源", "右上の歯車にある「予定の通知音」で、落ち着いた標準音または自分の音声ファイルを選べます。")
    story.extend([
        data_table([
            ("やわらぎ", "澄んだチャイム。初期設定"),
            ("深い雫", "静かに響く低い音"),
            ("小鈴", "控えめな鈴の音"),
            ("朝露のピアノ", "やわらかな短いピアノ"),
            ("木漏れ日のカリンバ", "穏やかな木の音色"),
            ("音なし", "ポップアップだけを表示"),
        ], styles, 47 * mm),
        Spacer(1, 4 * mm),
        step(1, "音を選んで試聴", "標準音または設定済みの自分の音を選び、「選択中の音を試聴」で確認します。もう一度押すと停止します。", styles),
        step(2, "音量と再生秒数を調整", "音量は0～100%、通知で鳴らす長さは3～60秒で調整します。初期値は12秒です。試聴中は停止ボタンを押すまで再生します。", styles),
        step(3, "自分の音源を使う", "「ファイルを選ぶ」から15MBまでのMP3、M4A、AAC、WAV、OGG、Opus、FLAC、MIDIを選びます。選択したファイルはdata/notification-soundsへコピーされます。", styles),
        Spacer(1, 4 * mm),
        two_cards([
            ("MIDIの音色", "Koyomado内蔵の穏やかな音色で再生するため、元の楽器・音源とは異なる場合があります。"),
            ("再生できない場合", "拡張子だけ変更したファイルは使用できません。一般音声はWindows WebView2の対応コーデックにも依存します。別形式へ変換して試してください。"),
        ], styles, [PURPLE_PALE, ROSE_PALE]),
        Spacer(1, 4 * mm),
        card("標準音の利用条件", "同梱5音はCC0 1.0素材をもとにしています。出典とKoyomadoでの編集内容は、配布物のNOTIFICATION_SOUNDS_CC0.txtとTHIRD_PARTY_NOTICES.mdで確認できます。", styles, GREEN_PALE),
        PageBreak(),
    ])

    # 13: data and update
    page_title(story, styles, "DATA AND UPDATE", "データ保存・持ち運び・更新", "予定と設定は暗号化せず、koyomado.exeと同じ場所のdataフォルダーへ保存します。Googleの更新トークンだけはWindows資格情報マネージャーへ保存します。")
    story.extend([
        data_table([
            ("calendar-data.json", "予定、日時、繰り返し、リマインダー、削除済み予定、外観、通知音、Google接続設定"),
            ("calendar-data.backup.json", "予定データを更新する直前のバックアップ"),
            ("calendar-data.v1～v4.backup.json", "旧形式からversion 5へ移行する前の予定データ（移行時のみ）"),
            ("notification-sounds", "自分で設定した通知音。dataフォルダーと一緒に持ち運びます"),
            ("window-state.json", "モニター構成ごとのウィンドウ位置とサイズ"),
            ("window-state.backup.json", "位置情報を更新する直前のバックアップ"),
            ("window-state.v1.backup.json", "旧形式から移行する前の位置情報（移行時のみ）"),
            ("Windows資格情報", "Googleの更新トークン。PCごとに保存され、フォルダーには含まれません"),
        ], styles, 58 * mm),
        Spacer(1, 4 * mm),
        two_cards([
            ("USBメモリ", "Koyomadoフォルダー全体をコピーします。取り外す前にKoyomadoを完全終了してください。"),
            ("Google Drive", "同期完了後に起動し、同じフォルダーを複数PCから同時に開かないでください。競合の自動解決は行いません。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        p("新しい版へ更新する", styles["h2"]),
        step(1, "Koyomadoを終了", "タスクバーのみなら×で終了。トレイを使う設定なら、トレイアイコンを右クリックして「終了」を選びます。", styles),
        step(2, "dataをバックアップ", "現在のKoyomadoフォルダー内のdataフォルダーを、別の安全な場所へコピーします。", styles),
        step(3, "新しいZIPを展開", "新しいフォルダーへ「すべて展開」します。", styles),
        step(4, "dataを引き継ぐ", "古いKoyomadoフォルダーのdataフォルダーを、新しいKoyomadoフォルダーへコピーします。", styles),
        step(5, "起動して確認", "koyomado.exeを起動し、予定・背景・位置を確認します。自動起動は新しい場所からONにし直し、Google連携は移動先PCで再認証します。", styles),
        Spacer(1, 3 * mm),
        card("大切な注意", "dataフォルダーを削除したり、新しい空のdataだけを残したりすると、予定を引き継げません。アプリの更新前には必ずフォルダーごとバックアップしてください。", styles, ROSE_PALE),
        PageBreak(),
    ])

    # 12: Google overview
    page_title(story, styles, "GOOGLE CALENDAR", "Google連携のしくみ", "Google連携は任意機能で、初期状態はOFFです。利用者が有効にした場合だけGoogleへ通信します。")
    story.extend([
        two_cards([
            ("連携しない", "Koyomadoは従来どおり完全にローカルで動作します。Googleへの通信、ログイン、API設定は不要です。"),
            ("連携する", "利用者自身のGoogle CloudプロジェクトとOAuthクライアントを使い、選んだカレンダーと双方向同期します。"),
        ], styles, [GREEN_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        p("同期する内容", styles["h2"]),
        data_table([
            ("予定", "予定名、開始・終了日時、終日、場所、メモ、繰り返し、リマインダー、削除"),
            ("複数日", "終日の連休・出張、日をまたぐ時刻付き予定を維持"),
            ("同期先", "ローカルのみ、特定アカウント、接続中の全アカウントから予定ごとに選択"),
            ("アカウント", "最大3件。各アカウントで同期するカレンダーを1つ選択"),
            ("予定の色", "Koyomado内の見た目として保持。Googleの色とは同期しません"),
        ], styles, 37 * mm),
        Spacer(1, 4 * mm),
        p("データと認証情報", styles["h2"]),
        bullet("予定はGoogle Calendar APIと利用者のPC間で直接送受信し、Y-TECのサーバーを経由しません。", styles),
        bullet("OAuthクライアントID、クライアントシークレット、プロジェクトIDはcalendar-data.jsonへ保存します。", styles),
        bullet("Googleの更新トークンはWindows資格情報マネージャーへ保存し、ポータブルフォルダーやGoogle Driveには入れません。", styles),
        card("通信を止める", "歯車でGoogleカレンダー連携をOFFにすると自動同期を停止します。アカウントの「接続解除」では認証情報と同期リンクを削除し、取り込み済み予定はローカル予定として残します。", styles, PURPLE_PALE),
        Spacer(1, 3 * mm),
        p("最初の接続は、この7段階", styles["h2"]),
        roadmap([
            (1, "専用プロジェクト", "Koyomado用を選ぶ"),
            (2, "Calendar API", "有効化を確認"),
            (3, "Branding", "アプリ名とメール"),
            (4, "対象", "Externalと連絡先"),
            (5, "Desktop client", "JSONをダウンロード"),
            (6, "本番環境", "アプリを公開"),
            (7, "Koyomado", "JSON読込と接続"),
        ], styles),
        PageBreak(),
    ])

    # 13: personal-use production policy
    page_title(story, styles, "GOOGLE BEFORE START", "常用設定は「In production」一本です", "利用者自身のOAuthプロジェクトを個人利用として本番環境へ切り替えます。公開サイトの準備やGoogleへの検証申請は行いません。")
    story.extend([
        data_table([
            ("採用する設定", "Google Auth Platformの「対象」で「アプリを公開」を押し、公開ステータスを「In production」にします。"),
            ("採用しない設定", "Testingは動作確認用です。Calendar権限を使う外部アプリでは更新トークンが原則7日で期限切れになるため、常用しません。"),
        ], styles, 34 * mm),
        Spacer(1, 4 * mm),
        card("個人利用なら検証申請は不要です", "利用者が自分のGoogle Cloudプロジェクトを作り、自分や身近な少人数だけで使う場合は、OAuth検証を申請せずにIn productionへ切り替えられます。ホームページ、プライバシーポリシー、承認済みドメイン、Y-TECのURLは入力しません。", styles, GREEN_PALE),
        Spacer(1, 4 * mm),
        two_cards([
            ("初回の警告", "未確認アプリの警告が出る場合があります。自分で作成したプロジェクト名と要求権限を確認したときだけ詳細から続行します。"),
            ("100ユーザー上限", "未検証プロジェクトには生涯100新規ユーザーの上限があります。利用者ごとに自分のプロジェクトを作り、最大3アカウントを接続する本方式では通常影響しません。"),
        ], styles, [SAND_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        url_link("Google公式 - 公開ステータスとTestingの期限", GOOGLE_AUDIENCE_HELP_URL, styles),
        url_link("Google公式 - 個人利用で検証が不要な場合", GOOGLE_VERIFICATION_HELP_URL, styles),
        PageBreak(),
    ])

    # 14: Google Cloud project and API
    page_title(story, styles, "GOOGLE CLOUD 1", "プロジェクト作成とCalendar API", "以下は2026年8月時点のGoogle Cloud画面名です。表示名が変わった場合は、近い名称の項目を選んでください。")
    story.extend([
        step(1, "Google Cloud Consoleを開く", f'<link href="{GOOGLE_CONSOLE_URL}" color="#5f5278">{GOOGLE_CONSOLE_URL}</link>へ、連携に使うGoogleアカウントでログインします。', styles),
        step(2, "プロジェクトを作成", "「新しいプロジェクト」を開き、名前をKoyomado Personalなどにします。プロジェクトIDは自動生成のままで構いません。作成後、画面上部でそのプロジェクトを選びます。", styles),
        step(3, "Google Calendar APIを有効化", "APIライブラリでGoogle Calendar APIを開き、「有効にする」を押します。似た名前のCalDAV APIは選びません。", styles),
        Spacer(1, 2 * mm),
        url_link("プロジェクト作成", GOOGLE_PROJECT_CREATE_URL, styles),
        url_link("Google Calendar API", GOOGLE_CALENDAR_API_URL, styles),
        Spacer(1, 2.5 * mm),
        screenshot_pair(
            ("oauth/01-google-project-create.png", "図1  名前を入力して「作成」。画面上部に割り当て数の注意が出ても、作成できる残数があれば進められます。"),
            ("oauth/02-enable-calendar-api.png", "図2  Google Calendar APIの画面で「有効にする」を押します。"),
            styles,
        ),
        Spacer(1, 3 * mm),
        card("APIキーは作りません", "Koyomadoが使うのはAPIキーではなく、デスクトップアプリ用のOAuth 2.0クライアントです。利用者自身のプロジェクトを使うため、Y-TEC共通キーやY-TECへのAPI利用料はありません。Google Cloudの規約、割り当て、ほかに有効化したサービスの費用は利用者自身で管理してください。", styles, SAND_PALE),
        PageBreak(),
    ])

    # 15: Google Auth Platform start and app information
    page_title(story, styles, "GOOGLE CLOUD 2", "Google Auth Platformを開始する", "Calendar APIを有効にしたプロジェクトで、OAuth同意画面の設定を始めます。")
    story.extend([
        step(1, "Google Auth Platformの概要を開く", "正しいプロジェクトが選ばれていることを確認し、「開始」または「構成を開始」を押します。", styles),
        step(2, "アプリ名を入力", "アプリ名はKoyomado。ユーザーサポートメールは自分のGoogleアカウントを選びます。", styles),
        Spacer(1, 2 * mm),
        url_link("Google Auth Platform 概要", GOOGLE_AUTH_OVERVIEW_URL, styles),
        Spacer(1, 2.5 * mm),
        screenshot_figure("oauth/03-auth-platform-start.png", "図3  OAuthクライアントがまだ無い状態。右側の「開始」から設定を始めます。", styles),
        Spacer(1, 2.5 * mm),
        screenshot_figure("oauth/04-oauth-app-info.png", "図4  アプリ名はKoyomado。メール欄は必ず自分のアカウントを選びます（説明書では非表示）。", styles),
        PageBreak(),
    ])

    # 16: audience and contact information
    page_title(story, styles, "GOOGLE CLOUD 3", "対象と連絡先を設定する", "個人のGoogleアカウントで使う場合は通常「外部」を選び、連絡先には自分のメールアドレスを設定します。")
    story.extend([
        step(1, "対象は「外部」", "個人のGoogleアカウントでは「外部（External）」を選びます。Workspace組織内だけで使う場合は、管理者方針に応じて「内部」を選べることがあります。", styles),
        step(2, "連絡先情報", "GoogleからOAuth設定に関する通知を受け取る自分のメールアドレスを入力します。一般利用者へ公開する連絡先として扱われる場合があります。", styles),
        Spacer(1, 2 * mm),
        url_link("対象（Audience）", GOOGLE_AUTH_AUDIENCE_URL, styles),
        Spacer(1, 2.5 * mm),
        screenshot_figure("oauth/05-oauth-audience-external.png", "図5  個人利用は「外部」を選択して「次へ」。", styles),
        Spacer(1, 2.5 * mm),
        screenshot_figure("oauth/06-oauth-contact-email.png", "図6  デベロッパー連絡先へ自分のメールアドレスを入力（説明書では非表示）。", styles),
        PageBreak(),
    ])

    # 17: policy and OAuth client form
    page_title(story, styles, "GOOGLE CLOUD 4", "規約を確認し、Desktopクライアントを作る", "Koyomadoが必要とするのは「デスクトップ アプリ」用のOAuthクライアントです。APIキーやウェブアプリ用クライアントではありません。")
    story.extend([
        step(1, "API利用規約を確認", "表示されたGoogle APIサービスのユーザーデータポリシーを読み、同意できる場合だけチェックして作成します。", styles),
        step(2, "「クライアント」を開く", "「OAuth クライアントを作成」を押し、アプリケーションの種類で「デスクトップ アプリ」を選びます。名前はKoyomado Desktopなどで構いません。", styles),
        url_link("OAuthクライアント", GOOGLE_AUTH_CLIENTS_URL, styles),
        Spacer(1, 2.5 * mm),
        screenshot_figure("oauth/07-oauth-user-data-policy.png", "図7  規約を確認し、同意する場合だけチェックして「作成」。", styles),
        Spacer(1, 2.5 * mm),
        screenshot_figure("oauth/08-create-desktop-client.png", "図8  種類は必ず「デスクトップ アプリ」。名前は自分が判別しやすいもので構いません。", styles),
        PageBreak(),
    ])

    # 18: JSON download and production preparation
    page_title(story, styles, "GOOGLE CLOUD 5", "JSONを保存し、「対象」を開く", "OAuthクライアント作成後にJSONを保存します。次に公開ステータスを切り替えるため、Google Auth Platformの「対象」を開きます。")
    story.extend([
        step(1, "JSONをダウンロード", "作成完了ダイアログの「JSONをダウンロード」を押し、あとで分かる場所へ保存します。クライアントIDとクライアントシークレットは公開しません。", styles),
        step(2, "「対象」を開く", "左メニューの「対象」を開きます。本手順ではブランディングのホームページ、プライバシーポリシー、承認済みドメインは設定しません。", styles),
        Spacer(1, 2.5 * mm),
        screenshot_figure("oauth/09-download-oauth-json.png", "図9  IDとシークレットは説明書では非表示。下部からJSONをダウンロードします。", styles),
        Spacer(1, 3 * mm),
        two_cards([
            ("正しいJSON", "client_secret_...jsonという名前のDesktop app用OAuth JSONです。"),
            ("公開せずバックアップ", "完全なシークレットを含むJSONは作成時にだけ取得できます。GitHub、Web、メールへ載せず、安全な場所へバックアップします。紛失時はシークレットのローテーションが簡単です。"),
        ], styles, [GREEN_PALE, ROSE_PALE]),
        Spacer(1, 2.5 * mm),
        url_link("対象（Audience）", GOOGLE_AUTH_AUDIENCE_URL, styles),
        PageBreak(),
    ])

    # 19: publish to production confirmation
    page_title(story, styles, "GOOGLE CLOUD 6", "公開ステータスを本番環境へ変更する", "「対象」で公開ステータスをIn productionへ変更します。この切り替えはGoogleのOAuth検証申請とは別です。")
    story.extend([
        step(1, "「対象」を開く", "公開ステータスが「テスト」と表示されていることを確認し、「アプリを公開」を押します。", styles),
        screenshot_figure("oauth/11-publish-to-production.png", "図10  「アプリを公開」を押します。Koyomadoを常用する場合はTestingのままにしません。", styles),
        Spacer(1, 2.5 * mm),
        step(2, "確認内容を読む", "未確認アプリの警告や新規ユーザー数の上限に関する説明を読み、「確認」を押します。", styles),
        screenshot_figure("oauth/12-confirm-production.png", "図11  内容を確認して「確認」。これはGoogleの検証申請そのものではありません。", styles),
        PageBreak(),
    ])

    # 20: production complete
    page_title(story, styles, "GOOGLE CLOUD 7", "「本番環境」になったことを確認する", "公開ステータスが本番環境へ変われば、Testing特有の7日間制限を避ける準備は完了です。")
    story.extend([
        screenshot_figure("oauth/13-production-complete.png", "図12  「公開ステータス: 本番環境」を確認します。", styles),
        Spacer(1, 4 * mm),
        two_cards([
            ("未確認アプリ警告", "本番環境にしてもOAuth検証は行わないため、接続時に警告が出ることがあります。自分で作成したプロジェクトであることを確認します。"),
            ("検証申請はしません", "このOAuthプロジェクトを使うのは作成者本人と接続する少数のアカウントだけです。Koyomadoの標準手順ではGoogleへの検証申請を行いません。"),
        ], styles, [SAND_PALE, SKY_PALE]),
        Spacer(1, 4 * mm),
        url_link("公開ステータス", GOOGLE_AUTH_AUDIENCE_URL, styles),
        url_link("検証が必要か確認", GOOGLE_VERIFICATION_HELP_URL, styles),
        card("ここからKoyomadoへ戻ります", "ダウンロードしたOAuth JSONをKoyomadoへ読み込みます。JSONは公開・共有せず、自分のPCでだけ使用してください。", styles, GREEN_PALE),
        PageBreak(),
    ])

    # 21: Koyomado JSON loading
    page_title(story, styles, "KOYOMADO CONNECT 1", "KoyomadoへJSONを読み込む", "Google CloudからダウンロードしたDesktop app用JSONを、Koyomadoの設定画面へ読み込みます。")
    story.extend([
        compact_steps([
            (1, "歯車を開く", "「表示と起動の設定」を開きます。"),
            (2, "Google連携をON", "ONにしたときだけGoogle連携設定が表示されます。"),
            (3, "JSONを選択", "ダウンロードしたclient_secret_...jsonを選びます。"),
            (4, "読込完了を確認", "ボタンが「JSONを読み直す」へ変わり、「OAuthクライアント設定を読み込みました」と表示されれば成功です。"),
        ], CONTENT_W, styles),
        Spacer(1, 3 * mm),
        screenshot_pair(
            ("oauth/14-koyomado-json-select.png", "図13  Googleカレンダー連携をONにして「JSONを選択」。"),
            ("oauth/15-koyomado-json-loaded.png", "図14  読込後は「JSONを読み直す」に変わります。"),
            styles,
        ),
        Spacer(1, 4 * mm),
        two_cards([
            ("JSONを読めない", "Desktop app用JSONか確認します。APIキー、サービスアカウント、Web application用JSONは使用できません。"),
            ("JSONの保管", "クライアントシークレットを含むため、GitHub、Webサイト、問い合わせメール、公開スクリーンショットへ載せないでください。"),
        ], styles, [SAND_PALE, ROSE_PALE]),
        PageBreak(),
    ])

    # 22: browser authorization and connection result
    page_title(story, styles, "KOYOMADO CONNECT 2", "ブラウザーで許可し、接続完了を確認する", "Koyomadoの「アカウントを接続」を押すと既定ブラウザーが開きます。認証が終わるまでKoyomadoを閉じないでください。")
    story.extend([
        roadmap([
            (1, "アカウント選択", "接続するGoogleアカウント"),
            (2, "未確認警告", "自分のプロジェクトか確認"),
            (3, "次へ", "メールアドレスの利用を確認"),
            (4, "すべて選択", "一覧参照と予定編集"),
            (5, "続行", "Koyomadoへ戻る"),
            (6, "接続結果", "Koyomadoで1/3件を確認"),
            (7, "カレンダー", "同期先を1つ選択"),
            (8, "既定保存先", "普段使うアカウントを選択"),
        ], styles),
        Spacer(1, 3 * mm),
        card("未確認アプリの警告が出たら", "自分で作成したGoogle Cloudプロジェクト名、選択したGoogleアカウント、要求権限がこの説明書と一致する場合だけ「詳細」-「Koyomado（安全ではないページ）に移動」へ進みます。心当たりのないクライアントでは中止してください。", styles, SAND_PALE),
        Spacer(1, 3 * mm),
        data_table([
            ("許可する権限1", "登録しているGoogleカレンダー一覧の参照"),
            ("許可する権限2", "すべてのカレンダーの予定の表示と編集"),
            ("成功画面", "ブラウザーに「Koyomadoと接続しました」と表示"),
            ("Koyomado側", "接続アカウントが1/3件になり、同期するカレンダーと既定の保存先を選択可能"),
        ], styles, 42 * mm),
        Spacer(1, 3 * mm),
        card("3分以内に完了してください", "Koyomadoのローカル認証待受には3分の制限があります。時間切れになった場合は、Koyomadoへ戻って「アカウントを接続」を押し、最初から認証し直してください。ブラウザーの成功表示だけでなく、Koyomadoのアカウント件数まで確認します。", styles, ROSE_PALE),
        PageBreak(),
    ])

    # 23: account and sync operation
    page_title(story, styles, "GOOGLE SYNC", "アカウント・同期先・同期操作", "接続後に対象カレンダーを確認し、新規予定の既定値と予定ごとの送信先を選びます。")
    story.extend([
        step(1, "同期するカレンダーを選ぶ", "接続直後にアカウントの「同期するカレンダー」を選びます。最初の同期後は誤結合を防ぐため選択をロックします。変更するときは接続解除後に接続し直します。", styles),
        step(2, "アカウントの同期をON", "アカウントカードの「このアカウントと同期」をONにします。不要なアカウントだけ一時停止できます。", styles),
        step(3, "新規予定の既定保存先を決める", "設定画面の「新しい予定の既定の保存先」で、いつも使う1件、複数件、または「すべて選択」を指定します。何も選ばなければローカルのみです。", styles),
        step(4, "予定ごとに保存先を確認", "予定追加・編集画面では既定値が自動選択されます。その予定だけ解除・追加できます。Googleから取得した予定は元アカウントへの同期を解除できません。", styles),
        step(5, "初回同期を確認", "設定画面の「今すぐ同期」を押し、最終同期日時とエラー表示を確認します。必要なら「Koyomado接続テスト」など明確なテスト予定を1件だけ作り、往復確認後にKoyomadoとGoogleの両方から削除します。", styles),
        Spacer(1, 4 * mm),
        p("自動同期のタイミング", styles["h2"]),
        data_table([
            ("起動・再表示", "アプリ起動時、トレイなどから画面へ戻したとき"),
            ("定期", "アプリが表示されている間、約60秒ごと"),
            ("予定変更直後", "追加、編集、削除、ドラッグ移動、コピー後"),
            ("手動", "上部の同期ボタン、または設定の「今すぐ同期」"),
        ], styles, 43 * mm),
        Spacer(1, 4 * mm),
        card("自宅PCと会社PCで使う場合", "各PCへ別々のKoyomadoフォルダーを置き、それぞれで同じGoogleアカウントへ接続する方法が安全です。Googleカレンダーを共通の同期元として使います。同じGoogle Drive上のKoyomadoフォルダーを2台から同時起動するとcalendar-data.jsonが競合するため避けてください。更新トークンはPCごとのWindows資格情報へ保存されるので、各PCで再認証します。", styles, SKY_PALE),
        PageBreak(),
    ])

    # 24: Google conflicts and troubleshooting
    page_title(story, styles, "GOOGLE TROUBLE", "競合・解除・接続トラブル", "同期で勝手に予定を消さないため、両側編集を検出したときは内容を2件に分けて残します。")
    story.extend([
        two_cards([
            ("競合が起きた", "同じ予定をKoyomadoとGoogleで前回同期後に変更すると、両方の内容を別予定として保存し、編集画面に競合表示を出します。内容を確認し、必要な方を残してください。"),
            ("接続を解除", "アカウントカードの「接続解除」を選びます。Googleへのトークン失効を試み、Windows資格情報と同期リンクを削除します。取り込み済み予定はローカルへ残ります。"),
        ], styles, [ROSE_PALE, GREEN_PALE]),
        Spacer(1, 4 * mm),
        data_table([
            ("7日ほどで再認証", "公開ステータスがTestingの可能性。「対象」でIn productionへ切り替え、Koyomadoで一度接続解除して認証し直す"),
            ("未確認アプリの警告", "自分で作成したプロジェクト名・Googleアカウント・要求権限を確認。心当たりがなければ中止"),
            ("JSONを読めない", "Desktop app用JSONか確認。APIキーのJSONやWeb application用は使用不可"),
            ("JSONを紛失した", "Google Auth Platformでクライアントシークレットをローテーションし、新しいJSONをダウンロード。できない場合はDesktopクライアントを作り直します"),
            ("ブラウザー後に戻らない", "Koyomadoを開いたまま再試行。ファイアウォールやセキュリティ製品のlocalhost通信も確認"),
            ("カレンダーを変えたい", "いったん接続解除し、再接続直後に対象カレンダーを選び直す"),
            ("予定が同期されない", "Google連携、アカウント同期、予定の同期先を確認して「今すぐ同期」"),
        ], styles, 48 * mm),
        Spacer(1, 4 * mm),
        card("安全と費用", "OAuth JSON、トークン、実予定を第三者へ送らないでください。Koyomadoは課金設定を追加せず、Y-TECの共通APIも使いません。ただしGoogle Cloudアカウント全体の設定と利用規約は利用者の責任で管理してください。", styles, SAND_PALE),
        Spacer(1, 4 * mm),
        p(f'<link href="{GOOGLE_USER_DATA_URL}" color="#5f5278">Google API Services User Data Policy</link>', styles["link"]),
        PageBreak(),
    ])

    # 25: trouble and reference
    page_title(story, styles, "HELP", "困ったとき・早見表", "画面に予定が見えない場合は、まず月、日付、タスクバーまたはトレイ、dataフォルダーの順に確認してください。")
    story.extend([
        data_table([
            ("画面が見つからない", "タスクバーを確認。トレイを使う設定ではKoyomadoアイコンを左クリックし、隠れているアイコンも確認します。"),
            ("予定が見えない", "表示中の年月を確認し、予定のある日付をクリックして日の一覧を開きます。"),
            ("記念日が翌年に出ない", "繰り返し周期が「毎年」か確認。2月29日はうるう年だけ表示されます。"),
            ("終了日時が戻った", "開始時刻を変えると終了は1時間後へ再設定されます。開始を決めた後に終了日時を変更します。"),
            ("貼り付けが選べない", "先に予定を右クリックして「内容をコピー」を選びます。"),
            ("通知が出ない", "Koyomadoが起動中か、予定の通知時刻、音なし設定、音量を確認。スリープ中や終了中の通知は後からまとめて表示しません。"),
            ("起動時の位置がおかしい", "現在のモニター構成で最後に保存した位置へ戻ります。初めての構成や画面外の位置は自動で見える位置へ戻るため、希望の場所へ移動して終了し直してください。"),
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
        p("Koyomadoには、印刷、予定のPDF出力、アクセス解析、独自クラウドサーバーはありません。外部通信は利用者が任意で有効にするGoogleカレンダー連携だけです。予定データとOAuthクライアント設定は暗号化されないため、機密情報やパスワードの保存には使用しないでください。祝日は1970年から2050年までの内蔵データを使います。", styles["body"]),
        card("お問い合わせ前に用意すると役立つ情報", "Koyomadoのバージョン、Windowsのバージョン、発生した操作、表示されたメッセージ、再現手順。予定の本文や個人情報は送らないでください。", styles, SAND_PALE),
        Spacer(1, 5 * mm),
        p(f'<b>公式ページ</b><br/><link href="{OFFICIAL_URL}" color="#5f5278">{OFFICIAL_URL}</link><br/><br/><b>ソースコード</b><br/><link href="{SOURCE_URL}" color="#5f5278">{SOURCE_URL}</link><br/><br/>利用条件はApache License 2.0のLICENSE.txt、Google連携の扱いはPRIVACY.mdをご確認ください。', styles["link"]),
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
        default=ROOT / "docs" / "Koyomado操作説明書.pdf",
        help="出力するPDFファイル",
    )
    args = parser.parse_args()
    generate(args.output.resolve())
    print(f"操作説明書を作成しました: {args.output.resolve()}")


if __name__ == "__main__":
    main()
