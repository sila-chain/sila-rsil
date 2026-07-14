import type { SidebarItem } from "./types";

export const rsilCliSidebar: SidebarItem = {
    text: "rsil",
    link: "/cli/rsil",
    collapsed: false,
    items: [
        {
            text: "rsil node",
            link: "/cli/rsil/node"
        },
        {
            text: "rsil init",
            link: "/cli/rsil/init"
        },
        {
            text: "rsil init-state",
            link: "/cli/rsil/init-state"
        },
        {
            text: "rsil import",
            link: "/cli/rsil/import"
        },
        {
            text: "rsil import-era",
            link: "/cli/rsil/import-era"
        },
        {
            text: "rsil export-era",
            link: "/cli/rsil/export-era"
        },
        {
            text: "rsil dump-genesis",
            link: "/cli/rsil/dump-genesis"
        },
        {
            text: "rsil db",
            link: "/cli/rsil/db",
            collapsed: true,
            items: [
                {
                    text: "rsil db stats",
                    link: "/cli/rsil/db/stats"
                },
                {
                    text: "rsil db list",
                    link: "/cli/rsil/db/list"
                },
                {
                    text: "rsil db checksum",
                    link: "/cli/rsil/db/checksum",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil db checksum mdbx",
                            link: "/cli/rsil/db/checksum/mdbx"
                        },
                        {
                            text: "rsil db checksum static-file",
                            link: "/cli/rsil/db/checksum/static-file"
                        },
                        {
                            text: "rsil db checksum rocksdb",
                            link: "/cli/rsil/db/checksum/rocksdb"
                        }
                    ]
                },
                {
                    text: "rsil db copy",
                    link: "/cli/rsil/db/copy"
                },
                {
                    text: "rsil db diff",
                    link: "/cli/rsil/db/diff"
                },
                {
                    text: "rsil db get",
                    link: "/cli/rsil/db/get",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil db get mdbx",
                            link: "/cli/rsil/db/get/mdbx"
                        },
                        {
                            text: "rsil db get static-file",
                            link: "/cli/rsil/db/get/static-file"
                        },
                        {
                            text: "rsil db get rocksdb",
                            link: "/cli/rsil/db/get/rocksdb"
                        }
                    ]
                },
                {
                    text: "rsil db drop",
                    link: "/cli/rsil/db/drop"
                },
                {
                    text: "rsil db clear",
                    link: "/cli/rsil/db/clear",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil db clear mdbx",
                            link: "/cli/rsil/db/clear/mdbx"
                        },
                        {
                            text: "rsil db clear static-file",
                            link: "/cli/rsil/db/clear/static-file"
                        }
                    ]
                },
                {
                    text: "rsil db repair-trie",
                    link: "/cli/rsil/db/repair-trie"
                },
                {
                    text: "rsil db static-file-header",
                    link: "/cli/rsil/db/static-file-header",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil db static-file-header block",
                            link: "/cli/rsil/db/static-file-header/block"
                        },
                        {
                            text: "rsil db static-file-header path",
                            link: "/cli/rsil/db/static-file-header/path"
                        }
                    ]
                },
                {
                    text: "rsil db version",
                    link: "/cli/rsil/db/version"
                },
                {
                    text: "rsil db path",
                    link: "/cli/rsil/db/path"
                },
                {
                    text: "rsil db settings",
                    link: "/cli/rsil/db/settings",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil db settings get",
                            link: "/cli/rsil/db/settings/get"
                        },
                        {
                            text: "rsil db settings set",
                            link: "/cli/rsil/db/settings/set",
                            collapsed: true,
                            items: [
                                {
                                    text: "rsil db settings set v2",
                                    link: "/cli/rsil/db/settings/set/v2"
                                }
                            ]
                        }
                    ]
                },
                {
                    text: "rsil db prune-checkpoints",
                    link: "/cli/rsil/db/prune-checkpoints",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil db prune-checkpoints get",
                            link: "/cli/rsil/db/prune-checkpoints/get"
                        },
                        {
                            text: "rsil db prune-checkpoints set",
                            link: "/cli/rsil/db/prune-checkpoints/set"
                        }
                    ]
                },
                {
                    text: "rsil db stage-checkpoints",
                    link: "/cli/rsil/db/stage-checkpoints",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil db stage-checkpoints get",
                            link: "/cli/rsil/db/stage-checkpoints/get"
                        },
                        {
                            text: "rsil db stage-checkpoints set",
                            link: "/cli/rsil/db/stage-checkpoints/set"
                        }
                    ]
                },
                {
                    text: "rsil db account-storage",
                    link: "/cli/rsil/db/account-storage"
                },
                {
                    text: "rsil db state",
                    link: "/cli/rsil/db/state"
                },
                {
                    text: "rsil db migrate-v2",
                    link: "/cli/rsil/db/migrate-v2"
                }
            ]
        },
        {
            text: "rsil download",
            link: "/cli/rsil/download"
        },
        {
            text: "rsil snapshot-manifest",
            link: "/cli/rsil/snapshot-manifest"
        },
        {
            text: "rsil stage",
            link: "/cli/rsil/stage",
            collapsed: true,
            items: [
                {
                    text: "rsil stage run",
                    link: "/cli/rsil/stage/run"
                },
                {
                    text: "rsil stage drop",
                    link: "/cli/rsil/stage/drop"
                },
                {
                    text: "rsil stage dump",
                    link: "/cli/rsil/stage/dump",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil stage dump execution",
                            link: "/cli/rsil/stage/dump/execution"
                        },
                        {
                            text: "rsil stage dump storage-hashing",
                            link: "/cli/rsil/stage/dump/storage-hashing"
                        },
                        {
                            text: "rsil stage dump account-hashing",
                            link: "/cli/rsil/stage/dump/account-hashing"
                        },
                        {
                            text: "rsil stage dump merkle",
                            link: "/cli/rsil/stage/dump/merkle"
                        }
                    ]
                },
                {
                    text: "rsil stage unwind",
                    link: "/cli/rsil/stage/unwind",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil stage unwind to-block",
                            link: "/cli/rsil/stage/unwind/to-block"
                        },
                        {
                            text: "rsil stage unwind num-blocks",
                            link: "/cli/rsil/stage/unwind/num-blocks"
                        }
                    ]
                }
            ]
        },
        {
            text: "rsil p2p",
            link: "/cli/rsil/p2p",
            collapsed: true,
            items: [
                {
                    text: "rsil p2p header",
                    link: "/cli/rsil/p2p/header"
                },
                {
                    text: "rsil p2p body",
                    link: "/cli/rsil/p2p/body"
                },
                {
                    text: "rsil p2p rlpx",
                    link: "/cli/rsil/p2p/rlpx",
                    collapsed: true,
                    items: [
                        {
                            text: "rsil p2p rlpx ping",
                            link: "/cli/rsil/p2p/rlpx/ping"
                        }
                    ]
                },
                {
                    text: "rsil p2p bootnode",
                    link: "/cli/rsil/p2p/bootnode"
                },
                {
                    text: "rsil p2p enode",
                    link: "/cli/rsil/p2p/enode"
                }
            ]
        },
        {
            text: "rsil config",
            link: "/cli/rsil/config"
        },
        {
            text: "rsil prune",
            link: "/cli/rsil/prune"
        },
        {
            text: "rsil re-execute",
            link: "/cli/rsil/re-execute"
        }
    ]
};
