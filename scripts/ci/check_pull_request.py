#!/usr/bin/env python3
import argparse
import subprocess
from typing import List
import yaml
import json


def git(*args, cwd=None):
    """Helper function to use git from python


    Args:
        args: git commands and options
        cwd: current working directory information passthrough subprocess.run
                Defaults to None.
    Returns:
        str: output from stdout
    """
    git_cmd = ("git",) + args
    try:
        cp = subprocess.run(git_cmd, capture_output=True, cwd=cwd)
    except OSError as err:
        print(f"Unexpected {err=}, {type(err)=}")
        raise

    return cp.stdout.decode("utf-8").rstrip()


def get_shas(refspec: str) -> List[str]:
    """Returns SHA values of the commits for given refspec

    Args:
        refspec (str):  refspec value to be pass to git rev-list

    Returns:
        sha_list(List[str]): List of SHA values
    """
    return git("rev-list", refspec).split()


def get_components_from_commits(refspec: str) -> List[str]:
    """Get component list from the commits in refspec

    Args:
        refspec (str):  refspec value to be pass to git rev-list

    Returns:
        components(List[str]): List of components
    """
    shas = get_shas(refspec)

    with open(".git_components.yaml", "r") as f:
        components = yaml.load(f.read(), Loader=yaml.Loader)
        components = set(components.keys())
        all_components = set()
        for sha in shas:
            commit_components = [
                c.strip()
                for c in git(
                    "-c", "log.showSignature=false", "show", sha, "-s", "--format=%s"
                ).split(":")[:-1]
                if c.strip() in components
            ]
            all_components.update(commit_components)
        return list(all_components)

    return []


def get_component_owners(component_name: str) -> List[str]:
    """Get owners of the specified component

    Args:
        component_name (str): Name of the component to get owners

    Returns:
        owners (List[str]): List of owners for the specified component_name.
            Returns empty list if there is no owners specified.

    """
    with open(".git_components.yaml", "r") as f:
        components = yaml.load(f.read(), Loader=yaml.Loader)
        component = components[component_name]
        if component:
            return component["owners"]
        else:
            return []


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--components-between-ref",
        type=str,
        help="get components from commits in refspec",
    )
    parser.add_argument(
        "--get-component-owners",
        type=str,
        help="get the owners of components to remove the author from the output use <component>,<author> syntax as argument",
    )
    args = parser.parse_args()

    if args.components_between_ref:
        components = get_components_from_commits(args.components_between_ref)
        print("\n".join(components))
    elif args.get_component_owners:
        author = None
        if "," in args.get_component_owners:
            component, author = args.get_component_owners.split(",")
        else:
            component = args.get_component_owners
        owners = get_component_owners(component)
        if author and owners and author in owners:
            owners.remove(author)
        print(json.dumps(owners))
