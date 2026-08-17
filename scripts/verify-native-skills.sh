#!/usr/bin/env bash
set -euo pipefail

operator="skills/actuation-operation/SKILL.md"
extension="skills/actuation-extension/SKILL.md"

for skill in "$operator" "$extension"; do
  test -f "$skill"
  head -n 1 "$skill" | grep -qx -- '---'
  grep -q '^name:' "$skill"
  grep -q '^description:' "$skill"
  grep -q '^## Contract metadata' "$skill"
  grep -q '^## .*Procedure' "$skill"
done

grep -q 'actuation:operator' "$operator"
grep -q 'validateWorldBinding' "$operator"
grep -q 'isRootAgency' "$operator"
grep -q 'validateMetagencyGrant' "$operator"
grep -q 'validateDetermination' "$operator"
grep -q 'validateReturn' "$operator"
grep -q 'Skill available != Capability granted' "$operator"
grep -q 'world mutation' "$operator"

grep -q 'actuation:extension-developer' "$extension"
grep -q 'native-owner review' "$extension"
grep -q 'Factory Claim / Run' "$extension"

echo "Actuation native Skills: structural contract OK"
