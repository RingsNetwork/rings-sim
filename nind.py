import argparse
import json
import logging
import os
import pathlib
import uuid
from typing import cast

logger = logging.getLogger("nind")
base_dir = pathlib.Path(__file__).parent
output_format = "cmdline"

ROUTER_IMAGE = "bnsnet/router"
NODE_IMAGE = "bnsnet/node"
BUILDER_IMAGE = "bnsnet/node-builder"

try:
    from python_on_whales import docker
    from python_on_whales.components.container.cli_wrapper import Container
    from python_on_whales.components.volume.cli_wrapper import VolumeDefinition
except ImportError:
    logger.error("Please install python-on-whales")
    exit(1)


def init_logger(level):
    _print_to_stderr = logging.StreamHandler()
    _print_to_stderr.setFormatter(
        logging.Formatter("%(asctime)s [%(levelname)s] %(name)s: %(message)s")
    )

    logger = logging.getLogger("nind")
    logger.addHandler(_print_to_stderr)
    logger.setLevel(level)


def parse_args():
    parser = argparse.ArgumentParser(
        description="NIND(Node in Docker). Create node and configure NAT"
    )
    parser.add_argument(
        "-v",
        dest="verbose",
        action="count",
        default=0,
        help="Set logging level, default is WARNING, -v is INFO, -vv is DEBUG",
    )
    parser.add_argument(
        "-f",
        "--output-format",
        default=output_format,
        choices=["cmdline", "json"],
        help="Stdout format, use json while pipe to other process",
    )

    subparsers = parser.add_subparsers(dest="subcmd", required=True)

    build_image = subparsers.add_parser(
        "build_image", help="Build images for node and router"
    )
    build_image.add_argument(
        "-p",
        "--path",
        type=pathlib.Path,
        default="./docker",
        help="Path to the base directory contained build directories",
    )
    build_image.add_argument(
        "--builder",
        action="store_true",
        help="Export builder image for debug mode (it's super huge)",
    )

    create_nat = subparsers.add_parser("create_nat", help="Create a NAT")
    create_nat.add_argument(
        "--router-image",
        type=str,
        default=ROUTER_IMAGE,
        help="Image for router container",
    )
    create_nat.add_argument(
        "-n",
        "--name",
        type=str,
        help="Name for router container",
    )
    create_nat.add_argument(
        "-l",
        "--lan",
        type=str,
        help="NATed network",
    )
    create_nat.add_argument(
        "-w",
        "--wan",
        type=str,
        default="bridge",
        help="Outer network",
    )
    create_nat.add_argument(
        "-s",
        "--symmetric",
        action="store_true",
        help="Create a Symmetric NAT (default is Port Restricted Cone NAT)",
    )

    create_node = subparsers.add_parser("create_node", help="Create a node")
    create_node.add_argument(
        "-l",
        "--lan",
        type=str,
        required=True,
        help="NATed network",
    )
    create_node.add_argument(
        "-r",
        "--router",
        type=str,
        required=True,
        help="Router container",
    )
    create_node.add_argument(
        "--node-image",
        type=str,
        default=NODE_IMAGE,
        help="Image for node container",
    )
    create_node.add_argument(
        "-n",
        "--name",
        type=str,
        help="Name for node container",
    )
    create_node.add_argument(
        "-s",
        "--stun",
        type=str,
        help="STUN server url",
    )
    create_node.add_argument(
        "-k",
        "--key",
        type=str,
        help="ETH key",
    )
    create_node.add_argument(
        "-d",
        "--debug",
        action="store_true",
        help="Run with volumed codes, so that you can restart container to update running codes",
    )
    create_node.add_argument(
        "-p",
        "--publish",
        action="append",
        help="Ports to publish, same as the `-p` argument in the Docker CLI",
    )
    create_node.add_argument(
        "-e",
        "--env",
        action="append",
        help="Environment variable kv pair splited by `=`",
    )
    create_node.add_argument(
        "-c",
        "--code",
        default=base_dir / "docker/bns-node",
        help="Specify code directory or volume to mount for debug mode",
    )
    create_node.add_argument(
        "-m",
        "--code-mount-mode",
        help="Specify code mount mode for debug mode like docker `-v` argument in the Docker CLI",
    )
    create_node.add_argument("cmd", nargs="*")

    subparsers.add_parser("clean", help="Clean up all containers and networks")

    return parser.parse_args()


def nonce():
    return uuid.uuid4().hex[-8:]


def get_mac_ifname(container, mac):
    cmd = "ip -br link | awk '$3 ~ /'{mac}'/ {{print $1}}'".format(mac=mac)
    output = container.execute(["bash", "-c", cmd])
    return output.split("@")[0]


def build_image(args):

    if args.builder:
        p = args.path
        logger.info(f"Building image, path: {p}")
        # Do not use buildx to prevent sending huge tarball. Known issue: https://github.com/docker/buildx/issues/107
        os.system(f"docker build -t {BUILDER_IMAGE} --target builder {p}")
        return

    p = args.path / "bns-router"
    logger.info(f"Building image, path: {p}")
    docker.build(context_path=p, tags=[ROUTER_IMAGE], load=True)

    p = args.path
    logger.info(f"Building image, path: {p}")
    docker.build(context_path=p, tags=[NODE_IMAGE], load=True)


def create_nat(args):
    wan_nw = docker.network.list(filters={"name": args.wan})[0]

    if args.lan is None:
        lan_nw = docker.network.create(
            f"bns-nw-{nonce()}",
            labels={"operator": "nind"},
        )
    else:
        lan_nw = docker.network.list(filters={"name": args.lan})[0]

    if args.name is None:
        args.name = f"bns-router-{nonce()}"

    router = docker.container.create(
        args.router_image,
        name=args.name,
        cap_add=["NET_ADMIN"],
        networks=[lan_nw],
        sysctl={"net.ipv4.ip_forward": "1"},
        labels={"operator": "nind"},
    )
    docker.network.connect(wan_nw, router)
    router.start()
    router.reload()

    wan_ip = router.network_settings.networks[wan_nw.name].ip_address
    wan_mac = router.network_settings.networks[wan_nw.name].mac_address
    wan_ifname = get_mac_ifname(router, wan_mac)

    lan_ip = router.network_settings.networks[lan_nw.name].ip_address
    lan_mac = router.network_settings.networks[lan_nw.name].mac_address
    lan_ifname = get_mac_ifname(router, lan_mac)

    logger.info(f"Router Container ID {router.id}")
    logger.info(f"(wan {wan_nw.name}) Ifname {wan_ifname}")
    logger.info(f"(wan {wan_nw.name}) IP Address {wan_ip}")
    logger.info(f"(wan {wan_nw.name}) Mac Address {wan_mac}")
    logger.info(f"(lan {lan_nw.name}) Ifname {lan_ifname}")
    logger.info(f"(lan {lan_nw.name}) IP Address {lan_ip}")
    logger.info(f"(lan {lan_nw.name}) Mac Address {lan_mac}")
    logger.info("Configuring iptables...")

    lan_subnet = lan_nw.ipam.config[0]["Subnet"]
    if args.symmetric:
        cmds = [
            [
                "iptables-legacy",
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-s",
                lan_subnet,
                "-o",
                wan_ifname,
                "-j",
                "MASQUERADE",
                "--random",
            ],
            [
                "iptables-legacy",
                "-A",
                "FORWARD",
                "-i",
                wan_ifname,
                "-o",
                lan_ifname,
                "-m",
                "state",
                "--state",
                "RELATED,ESTABLISHED",
                "-j",
                "ACCEPT",
            ],
            [
                "iptables-legacy",
                "-A",
                "FORWARD",
                "-i",
                lan_ifname,
                "-o",
                wan_ifname,
                "-j",
                "ACCEPT",
            ],
        ]
        for cmd in cmds:
            router.execute(cmd)
    else:
        router.execute(
            [
                "iptables-legacy",
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-s",
                lan_subnet,
                "-o",
                wan_ifname,
                "-j",
                "SNAT",
                "--to-source",
                wan_ip,
            ]
        )

    if output_format == "cmdline":
        print(f"-l {lan_nw.name} -r {router.name}")
    elif output_format == "json":
        print(json.dumps({"lan": lan_nw.name, "router": router.name}))


def create_node(args):

    ###################################
    # Query and check network configs #
    ###################################

    router = next(iter(docker.container.list(filters={"name": args.router})), None)
    if router is None:
        logger.error(f"Cannot find router by name {args.router}")
        exit(1)

    lan_nw = next(iter(docker.network.list(filters={"name": args.lan})), None)
    if lan_nw is None:
        logger.error(f"Cannot find lan by name {args.lan}")
        exit(1)

    wan_nw_id = next(
        v.network_id
        for v in router.network_settings.networks.values()
        if v.network_id != lan_nw.id
    )
    wan_nw = docker.network.list(filters={"id": wan_nw_id})[0]
    wan_subnet = wan_nw.ipam.config[0]["Subnet"]

    router_ip = next(
        v.ipv4_address for k, v in lan_nw.containers.items() if k == router.id
    ).split("/")[0]

    #######################
    # Create node by args #
    #######################

    if args.name is None:
        args.name = f"bns-node-{nonce()}"

    if args.key is None:
        args.key = "".join([uuid.uuid4().hex + uuid.uuid4().hex])

    if args.stun is None:
        logger.info(
            "Stun server not provided, try finding locally by container name `coturn`"
        )
        coturn = next(iter(docker.container.list(filters={"name": "coturn"})), None)

        if coturn is None:
            logger.error("Stun server not found")
            exit(0)
        else:
            args.stun = coturn.network_settings.networks["bridge"].ip_address

    if ":" not in args.stun:
        args.stun = f"{args.stun}:3478"
    if not args.stun.startswith("stun://"):
        args.stun = f"stun://{args.stun}"

    volumes = []

    if args.debug:
        args.node_image = BUILDER_IMAGE
        args.name = f"{args.name}-debug"
        args.cmd = args.cmd or ["cargo", "run", "--", "run", "-b", "0.0.0.0:50000"]

        if args.code_mount_mode == "ro":
            logger.error("Cannot use readonly mode, cargo will manipulate files")
            exit(1)
        elif args.code_mount_mode is None:
            vlm = (args.code, "/src/bns-node")
        else:
            vlm = (args.code, "/src/bns-node", args.code_mount_mode)
        volumes = [cast(VolumeDefinition, vlm)]

    elif args.node_image == NODE_IMAGE:
        args.cmd = args.cmd or ["bns-node", "run", "-b", "0.0.0.0:50000"]

    logger.debug(f"Args: {args}")

    node = cast(
        Container,
        docker.container.run(
            args.node_image,
            args.cmd,
            name=args.name,
            detach=True,
            cap_add=["NET_ADMIN"],
            networks=[lan_nw],
            labels={"operator": "nind"},
            envs={
                "ICE_SERVERS": args.stun,
                "ETH_KEY": args.key,
                "RUST_BACKTRACE": "1",
                **dict(kv.split("=") for kv in args.env or []),
            },
            publish=[p.rsplit(":", 1) for p in args.publish or []],
            volumes=volumes,
        ),
    )
    node.reload()
    node_ip = node.network_settings.networks[lan_nw.name].ip_address

    logger.info(f"Node Container ID {node.id}")
    logger.info(f"(lan {lan_nw.name}) IP Address {node_ip}")
    logger.info("Add route...")

    #############
    # Add route #
    #############

    cmd = ["ip", "route", "add", wan_subnet, "via", router_ip, "dev", "eth0"]
    try:
        node.execute(cmd)
    except Exception:
        logger.warning(f"Add route failed. To fix it, manually run: {' '.join(cmd)}")

    ##########
    # Output #
    ##########

    if output_format == "cmdline":
        print(f"{lan_nw.name} {node.name}")
    elif output_format == "json":

        pub_port = None
        first_port_mappings = next(iter(node.network_settings.ports.values()), None)
        if first_port_mappings:
            pub_port = int(first_port_mappings[0]["HostPort"])

        print(
            json.dumps(
                {
                    "name": node.name,
                    "router": router.name,
                    "lan": lan_nw.name,
                    "lan_ip": node_ip,
                    "pub_port": pub_port,
                    "key": args.key,
                }
            )
        )


def clean():
    for c in docker.container.list(all=True, filters={"label": "operator=nind"}):
        c.remove(force=True)
    docker.network.prune(filters={"label": "operator=nind"})


def main():
    args = parse_args()
    init_logger(max(10, 10 * (3 - args.verbose)))

    global output_format
    output_format = args.output_format

    if args.subcmd == "build_image":
        build_image(args)
    elif args.subcmd == "create_nat":
        create_nat(args)
    elif args.subcmd == "create_node":
        create_node(args)
    elif args.subcmd == "clean":
        clean()


if __name__ == "__main__":
    main()
