#include <algorithm>
#include <cstdint>
#include <limits>

#include "array_id_func.h"
#include "chain.h"
#include "contraction_graph.h"
#include "greedy_order.h"
#include "permutation.h"
#include "sort_arc.h"

static int compute_max_bag_size_of_order_local(
    const ArrayIDIDFunc& tail,
    const ArrayIDIDFunc& head,
    const ArrayIDIDFunc& order
) {
    auto inv_order = inverse_permutation(order);
    int current_tail = -1;
    int current_tail_up_deg = 0;
    int max_up_deg = 0;
    compute_chordal_supergraph(
        chain(tail, inv_order),
        chain(head, inv_order),
        [&](int x, int) {
            if (current_tail != x) {
                current_tail = x;
                if (max_up_deg < current_tail_up_deg) {
                    max_up_deg = current_tail_up_deg;
                }
                current_tail_up_deg = 0;
            }
            ++current_tail_up_deg;
        }
    );
    return max_up_deg + 1;
}

extern "C" int flowcutter_compute_treewidth_from_edges(
    std::uint32_t node_count,
    std::uint32_t edge_count,
    const std::uint32_t* edge_u,
    const std::uint32_t* edge_v,
    std::uint32_t* out_treewidth
) {
    if (out_treewidth == nullptr) {
        return -1;
    }
    if (node_count == 0 || edge_count == 0) {
        *out_treewidth = 0;
        return 0;
    }
    if (edge_u == nullptr || edge_v == nullptr) {
        return -2;
    }

    try {
        const int n = static_cast<int>(node_count);
        const int m = static_cast<int>(edge_count);
        ArrayIDIDFunc tail(2 * m, n);
        ArrayIDIDFunc head(2 * m, n);

        int arc = 0;
        for (int i = 0; i < m; ++i) {
            const auto u = edge_u[i];
            const auto v = edge_v[i];
            if (u >= node_count || v >= node_count) {
                return -3;
            }
            tail[arc] = static_cast<int>(u);
            head[arc] = static_cast<int>(v);
            ++arc;
            tail[arc] = static_cast<int>(v);
            head[arc] = static_cast<int>(u);
            ++arc;
        }

        auto p = sort_arcs_first_by_tail_second_by_head(tail, head);
        tail = chain(p, std::move(tail));
        head = chain(p, std::move(head));

        int best_bag_size = std::numeric_limits<int>::max();

        auto degree_order = compute_greedy_min_degree_order(tail, head);
        best_bag_size = std::min(
            best_bag_size,
            compute_max_bag_size_of_order_local(tail, head, degree_order)
        );

        // min-shortcut is much more expensive; keep the original cutoff.
        if (n < 10000) {
            auto shortcut_order = compute_greedy_min_shortcut_order(tail, head);
            best_bag_size = std::min(
                best_bag_size,
                compute_max_bag_size_of_order_local(tail, head, shortcut_order)
            );
        }

        if (best_bag_size <= 0 || best_bag_size == std::numeric_limits<int>::max()) {
            *out_treewidth = 0;
            return 0;
        }

        const int treewidth = std::max(1, best_bag_size - 1);
        *out_treewidth = static_cast<std::uint32_t>(treewidth);
        return 0;
    } catch (...) {
        return -4;
    }
}
