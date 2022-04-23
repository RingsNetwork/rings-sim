import argparse
import logging
import pathlib
import uuid

GET_IFNAME_CMD = "bash -c \"ip -br link | awk '$3 ~ /'{mac}'/ {{print $1}}'\""
ADD_NAT_RULE_CMD = "iptables-legacy -t nat -A POSTROUTING -s {lan_subnet} -o {wan_ifname} -j SNAT --to-source {wan_ip}"

logger = logging.getLogger("nind")


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
        "--verbose",
        action="store_true",
        help="Set logging level to DEBUG",
    )

    subparsers = parser.add_subparsers(dest="cmd", required=True)

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

    create_nat = subparsers.add_parser("create_nat", help="Create a NAT")
    create_nat.add_argument(
        "--router-image",
        type=str,
        default="bns/router",
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
        default="bns/node",
        help="Image for node container",
    )
    create_node.add_argument(
        "-n",
        "--name",
        type=str,
        help="Name for node container",
    )

    clean = subparsers.add_parser("clean", help="Clean up all containers and networks")

    return parser.parse_args()


def nonce():
    return uuid.uuid4().hex[-8:]


def get_mac_ifname(container, mac):
    cmd = GET_IFNAME_CMD.format(mac=mac)
    output = exec_run(container, cmd)
    return output.split("@")[0]


def exec_run(container, cmd):
    output = container.exec_run(cmd).output.decode()

    logger.debug(f"exec cmd: {cmd}")
    logger.debug(f"cmd output: {output or '[NoOutput]'}")

    return output


def build_image(client, args):
    p = str(args.path / "bns-router")
    logger.info(f"Building image, path: {p}")
    client.images.build(path=p, tag="bns/router", rm=True)

    p = str(args.path / "bns-node")
    logger.info(f"Building image, path: {p}")
    client.images.build(path=p, tag="bns/node", rm=True)


def create_nat(client, args):
    wan_nw = client.networks.list(names=[args.wan])[0]

    if args.lan is None:
        lan_nw = client.networks.create(
            f"bns-nw-{nonce()}",
            labels={"operator": "nind"},
        )
    else:
        lan_nw = client.networks.list(names=[args.lan])[0]

    if args.name is None:
        args.name = f"bns-router-{nonce()}"

    router = client.containers.create(
        args.router_image,
        name=args.name,
        cap_add=["NET_ADMIN"],
        network=lan_nw.id,
        labels={"operator": "nind"},
    )
    wan_nw.connect(router)
    router.start()
    router.reload()

    wan_ip = router.attrs["NetworkSettings"]["Networks"][wan_nw.name]["IPAddress"]
    wan_mac = router.attrs["NetworkSettings"]["Networks"][wan_nw.name]["MacAddress"]
    wan_ifname = get_mac_ifname(router, wan_mac)

    lan_ip = router.attrs["NetworkSettings"]["Networks"][lan_nw.name]["IPAddress"]
    lan_mac = router.attrs["NetworkSettings"]["Networks"][lan_nw.name]["MacAddress"]
    lan_ifname = get_mac_ifname(router, lan_mac)

    logger.info(f"Router Container ID {router.id}")
    logger.info(f"(wan {wan_nw.name}) Ifname {wan_ifname}")
    logger.info(f"(wan {wan_nw.name}) IP Address {wan_ip}")
    logger.info(f"(wan {wan_nw.name}) Mac Address {wan_mac}")
    logger.info(f"(lan {lan_nw.name}) Ifname {lan_ifname}")
    logger.info(f"(lan {lan_nw.name}) IP Address {lan_ip}")
    logger.info(f"(lan {lan_nw.name}) Mac Address {lan_mac}")
    logger.info("Configuring iptables...")

    lan_subnet = lan_nw.attrs["IPAM"]["Config"][0]["Subnet"]
    cmd = ADD_NAT_RULE_CMD.format(
        lan_subnet=lan_subnet, wan_ifname=wan_ifname, wan_ip=wan_ip
    )
    exec_run(router, cmd)

    print(f"-l {lan_nw.name} -r {router.name}")


def create_node(client, args):
    router = client.containers.get(args.router)
    lan_nw = client.networks.get(args.lan)

    wan_nw_id = next(
        v["NetworkID"]
        for v in router.attrs["NetworkSettings"]["Networks"].values()
        if v["NetworkID"] != lan_nw.id
    )
    wan_nw = client.networks.get(wan_nw_id)
    wan_subnet = wan_nw.attrs["IPAM"]["Config"][0]["Subnet"]

    router_ip = next(
        v["IPv4Address"]
        for k, v in lan_nw.attrs["Containers"].items()
        if k == router.id
    ).split("/")[0]

    if args.name is None:
        args.name = f"bns-node-{nonce()}"

    node = client.containers.run(
        args.node_image,
        name=args.name,
        detach=True,
        cap_add=["NET_ADMIN"],
        network=lan_nw.id,
        labels={"operator": "nind"},
    )
    node.reload()
    node_ip = node.attrs["NetworkSettings"]["Networks"][lan_nw.name]["IPAddress"]

    logger.info(f"Node Container ID {node.id}")
    logger.info(f"(lan {lan_nw.name}) IP Address {node_ip}")
    logger.info("Add route...")

    cmd = f"ip route add {wan_subnet} via {router_ip} dev eth0"
    exec_run(node, cmd)

    print(f"{lan_nw.name} {node.name}")


def clean(client):
    for c in client.containers.list(all=True, filters={"label": "operator=nind"}):
        c.remove(force=True)
    client.networks.prune(filters={"label": "operator=nind"})


def main():
    args = parse_args()
    init_logger(logging.DEBUG if args.verbose else logging.INFO)

    try:
        import docker
    except ImportError:
        logger.error("Please install docker-py")
        exit(1)

    try:
        client = docker.from_env()
    except Exception:
        logger.error("Please run docker daemon")
        exit(1)

    if args.cmd == "build_image":
        build_image(client, args)
    elif args.cmd == "create_nat":
        create_nat(client, args)
    elif args.cmd == "create_node":
        create_node(client, args)
    elif args.cmd == "clean":
        clean(client)


if __name__ == "__main__":
    main()
