from test.test_importlib import extension
import argparse
import cadquery as cq
import os
from pathlib import Path
import string
from xml.etree.ElementTree import tostring
from astroid.nodes import as_string


def main():
    parser = argparse.ArgumentParser(
                        prog="alphabet_squared",
                        description="Generate 3D letter intersections.",
    )
    parser.add_argument("font_name")
    parser.add_argument("-o", "--output", default="output")

    args = parser.parse_args()
    print(args.font_name, args.output)

    output_dir = Path(args.output)
    os.makedirs(output_dir, exist_ok=True)

    chars = string.ascii_uppercase + string.digits
    for char_1 in chars:
        for char_2 in chars:
            name = f"{char_1}{char_2}"
            print(f"Preparing shape {name}")
            shape_1 = create_letter_shape(args.font_name, char_1, angle_deg=0.0)
            shape_2 = create_letter_shape(args.font_name, char_2, angle_deg=-90.0)
            shape = shape_1.intersect(shape_2)
            path = output_dir / f"{name}.stl"
            shape.export(str(path))


def create_letter_shape(font: str, c: str, angle_deg: float) -> cq.Shape:
    shape = cq.Workplane("XZ").text(
        c,
        0.5,
        0.05,
        font=font,
        cut=False,
        combine=False,
        halign="right",
        valign="top",
    )
    return shape
