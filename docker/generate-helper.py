from argparse import ArgumentParser
from dataclasses import dataclass
from textwrap import dedent
from typing import Any, Self, Never


# https://doc.rust-lang.org/cargo/appendix/glossary.html#target
@dataclass(frozen=True)
class GoTarget:
    os: str
    arch: str
    variant: str | None = None

    def to_platform(self) -> str:
        variant = f"/{self.variant}" if self.variant is not None else ""
        return f"{self.os}/{self.arch}{variant}"

    @classmethod
    def from_any(cls, value: Any) -> Self:
        assert isinstance(value.os, str)
        assert isinstance(value.arch, str)
        assert isinstance(value.variant, str) or value.variant is None
        return cls(value.os, value.arch, value.variant or None)


@dataclass(frozen=True)
class Target:
    arch: str
    sub: str | None
    vendor: str
    sys: str
    abi: str | None

    def _unexpected_arch(self) -> Never:
        raise ValueError(f"unexpected architecture {self.arch}")

    def triple(self) -> str:
        sub = self.sub or ""
        return f"{self.arch}{sub}-{self.vendor}-{self.sys}-{self.abi}"

    def dpkg_arch(self) -> str:
        match self.arch:
            case "arm64" | "arm" | "aarch64":
                return "arm64"
            case "amd64" | "x86_64":
                return "amd64"
            case "i386" | "i686":
                return "i386"
            case "ppc64le" | "powerpc64le":
                return "ppc64el"
            case "s390x":
                return "s390x"
            case "mips64le" | "mips64el":
                return "mips64el"
            case _:
                self._unexpected_arch()

    def cc(self) -> str:
        match self.arch:
            case "x86_64" | "amd64" | "i386" | "i686":
                return "x86_64-linux-gnu"
            case "arm64" | "arm" | "aarch64":
                return "aarch64-linux-gnu"
            case "ppc64le" | "powerpc64le":
                return "powerpc64le-linux-gnu"
            case "s390x":
                return "s390x-linux-gnu"
            case "mips64le" | "mips64el":
                return "mips64el-linux-gnuabi64"
            case _:
                self._unexpected_arch()


TARGET_TABLE = {
    GoTarget("linux", "386"): Target("i686", None, "unknown", "linux", "gnu"),
    GoTarget("linux", "amd64"): Target("x86_64", None, "unknown", "linux", "gnu"),
    GoTarget("linux", "arm64"): Target("aarch64", None, "unknown", "linux", "gnu"),
    GoTarget("linux", "arm"): Target("aarch64", None, "unknown", "linux", "gnu"),
    GoTarget("linux", "arm", "v6"): Target("arm", None, "unknown", "linux", "gnueabi"),
    GoTarget("linux", "arm", "v7"): Target("arm", "v7", "unknown", "linux", "gnueabi"),
    GoTarget("linux", "ppc64le"): Target("powerpc64le", None, "unknown", "linux", "gnu"),
    GoTarget("linux", "mips64le"): Target("mips64el", None, "unknown", "linux", "gnuabi64"),
    GoTarget("linux", "s390x"): Target("s390x", None, "unknown", "linux", "gnu")
}


def generate_helper(target: Target) -> str:
    triple = target.triple()
    dpkg_arch = target.dpkg_arch()
    cc = target.cc()
    return dedent("""
    function dpkg_add_architecture() {{
        dpkg --add-architecture {dpkg_arch}
    }}

    function install_apt_deps() {{
        apt-get -qq update
        apt-get -qq install --no-install-recommends \\
            crossbuild-essential-{dpkg_arch} libssl-dev:{dpkg_arch}
    }}

    function add_target() {{
        rustup target add {triple}
    }}

    function print_config() {{
    cat << 'EOF'
    [build]
    target = "{triple}"
    [env]
    {triple_env_upper}_OPENSSL_LIB_DIR = "/usr/lib/{cc}"
    {triple_env_upper}_OPENSSL_INCLUDE_DIR = "/usr/include/{cc}"
    TARGET_CC = "{cc}-gcc"
    TARGET_CXX = "{cc}-g++"
    [target.{triple}]
    linker = "{cc}-gcc"
    EOF
    }}

    function extract_binary() {{
        cat "${{1}}/{triple}/release/main"
    }}

    case "${{1}}" in
        "dpkg_add_architecture" ) dpkg_add_architecture ;;
        "install_apt_deps" ) install_apt_deps ;;
        "add_target" ) add_target ;;
        "print_config" ) print_config ;;
        "extract_binary" ) extract_binary "${{2}}" ;;
        * ) exit 1 ;;
    esac
    """).format(
        triple=triple,
        dpkg_arch=dpkg_arch,
        cc=cc,
        triple_env_upper=triple.replace("-", "_").upper(),
    )


def main() -> None:
    parser = ArgumentParser("helper.py")
    parser.add_argument("--os", required=True)
    parser.add_argument("--arch", required=True)
    parser.add_argument("--variant", required=False)
    args = parser.parse_args()
    go_target = GoTarget.from_any(args)
    if go_target not in TARGET_TABLE:
        raise ValueError(f"unexpected platform '{go_target.to_platform()}'")
    target = TARGET_TABLE[go_target]
    print(generate_helper(target).strip())


if __name__ == "__main__":
    main()
