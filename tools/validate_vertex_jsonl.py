import json
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(1)


def validate_record(record: object, *, path: Path, line_no: int) -> None:
    if not isinstance(record, dict):
        fail(f"{path}: line {line_no}: record is not an object")

    contents = record.get("contents")
    if not isinstance(contents, list) or not contents:
        fail(f"{path}: line {line_no}: missing/invalid 'contents'")

    for i, item in enumerate(contents):
        if not isinstance(item, dict):
            fail(f"{path}: line {line_no}: contents[{i}] is not an object")

        role = item.get("role")
        if not isinstance(role, str) or not role:
            fail(f"{path}: line {line_no}: contents[{i}] missing/invalid 'role'")

        parts = item.get("parts")
        if not isinstance(parts, list) or not parts:
            fail(f"{path}: line {line_no}: contents[{i}] missing/invalid 'parts'")

        for j, part in enumerate(parts):
            if not isinstance(part, dict):
                fail(f"{path}: line {line_no}: contents[{i}].parts[{j}] is not an object")
            text = part.get("text")
            if not isinstance(text, str):
                fail(f"{path}: line {line_no}: contents[{i}].parts[{j}] missing/invalid 'text'")


def validate_file(path: Path) -> int:
    if not path.exists():
        fail(f"{path}: file does not exist")
    if path.stat().st_size == 0:
        fail(f"{path}: file is empty")

    count = 0
    with path.open("r", encoding="utf-8") as f:
        for line_no, raw in enumerate(f, start=1):
            line = raw.rstrip("\n")
            if not line.strip():
                fail(f"{path}: line {line_no}: empty line")
            try:
                record = json.loads(line)
            except json.JSONDecodeError as e:
                fail(f"{path}: line {line_no}: invalid JSON ({e})")
            validate_record(record, path=path, line_no=line_no)
            count += 1

    if count == 0:
        fail(f"{path}: no records")
    return count


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print("Usage: validate_vertex_jsonl.py <file.jsonl> [more.jsonl...]", file=sys.stderr)
        return 2

    exit_code = 0
    for arg in argv[1:]:
        path = Path(arg)
        try:
            count = validate_file(path)
            print(f"{path}: {count} records")
        except SystemExit:
            exit_code = 1
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
