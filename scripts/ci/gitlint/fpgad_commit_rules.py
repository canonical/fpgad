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
