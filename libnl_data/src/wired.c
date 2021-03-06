#include "../include/nl_data.h"

void print_and_exit(char *err) {
    fprintf(stderr, "%s: %s\n", PREFIX_ERROR, err);
    exit(EXIT_FAILURE);
}

bool is_operational(struct rtnl_link *link) {
    char        buf[BUF_SIZE];
    uint8_t     state;
    bool        r;

    r = false;
    memset(buf, 0, BUF_SIZE);
    state = rtnl_link_get_operstate(link);
    rtnl_link_operstate2str(state, buf, BUF_SIZE);
    if (strcmp(buf, "up") == 0) {
        r = true;
    }
    return r;
}

bool has_ip(struct nl_cache *cache, int if_index) {
    struct nl_object    *obj;
    struct rtnl_addr    *addr;
    int                 index;
    int                 family;

    if (nl_cache_is_empty(cache)) {
        return false;
    }
    for(obj = nl_cache_get_first(cache); obj != NULL; obj = nl_cache_get_next(obj)) {
        addr = (struct rtnl_addr *)obj;
        index = rtnl_addr_get_ifindex(addr);
        family = rtnl_addr_get_family(addr);
        if (index == if_index
            && (family == AF_INET || family == AF_INET6)
            && rtnl_addr_get_local(addr) != NULL) {
            return true;
        }
    }
    return false;
}

t_wired_data get_wired_data(char *interface) {
    t_wired_data        data;
    struct nl_sock      *sk;
    struct nl_cache     *cache;
    struct rtnl_link    *link;
    int                 if_index;

    if ((sk = nl_socket_alloc()) == NULL) {
        print_and_exit("nl_socket_alloc");
    }
    if (nl_connect(sk, NETLINK_ROUTE) != 0) {
        print_and_exit("nl_connect");
    }
    if (rtnl_addr_alloc_cache(sk, &cache) != 0) {
        print_and_exit("rtnl_addr_alloc_cache");
    }
    if (rtnl_link_get_kernel(sk, 0, interface, &link) < 0) {
        print_and_exit("interface not found");
    }
    if_index = rtnl_link_get_ifindex(link);
    data.is_carrying = rtnl_link_get_carrier(link);
    data.is_operational = is_operational(link);
    data.has_ip = has_ip(cache, if_index);
    nl_socket_free(sk);
    nl_cache_free(cache);
    nl_object_free((struct nl_object *)link);
    return data;
}