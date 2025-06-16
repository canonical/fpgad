# This file is part of fpgad, an application to manage FPGA subsystem together with device-tree and kernel modules.
#
# Copyright 2025 Canonical Ltd.
#
# SPDX-License-Identifier: GPL-3.0-only
#
# fpgad is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License version 3, as published by the Free Software Foundation.
#
# fpgad is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranties of MERCHANTABILITY, SATISFACTORY QUALITY, or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License along with this program.  If not, see http://www.gnu.org/licenses/.

from gitlint.rules import (
    RuleViolation,
    CommitMessageTitle,
    LineRule,
)
from gitlint.options import ListOption
import yaml


class TitleStartsWithComponent(LineRule):
    name = "title-starts-with-component"
    id = "UC3"
    target = CommitMessageTitle
    options_spec = [
        ListOption(
            "components",
            [],
            "Components should match, see components list in .git_components.yaml",
        )
    ]

    def validate(self, title, _commit):
        title_sections = title.split(":")
        if len(title_sections) < 2:
            return [
                RuleViolation(
                    self.id,
                    "Commit title does not follow <component>: <subject>",
                    title,
                )
            ]

        title_components = title_sections[:-1]
        with open(".git_components.yaml", "r") as f:
            components = yaml.load(f.read(), Loader=yaml.Loader)
            components = list(components.keys())
            components.extend(self.options["components"].value)
            for title_component in title_components:
                if title_component.strip() not in components:
                    return [
                        RuleViolation(
                            self.id,
                            f"{title_component} is not found in available components (see .git_components.yaml)",
                            title,
                        )
                    ]
