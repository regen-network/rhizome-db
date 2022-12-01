# RhizomeDB

This repo consists of two related projects `rhizome-trees`, a foundational data structure layer, and `rhizome-db`
(an experimental peer to peer database):

## `rhizome-trees`

A set of data structures which share the following properties:
* persistent like Clojure collections so that historical state can be maintained with arbitrary branching
* serializable to a variety of storage layers such as key values as well as directly to disk
* support merkle hashes and merkle proofs
* support selective pruning of historical state

These data structures are intended to support a variety of applications which can benefit from such data structures
including:
* distributed ledgers (blockchains)
* peer to peer databases
* any database which needs historical state

All the trees will be benchmarked comprehensively to allow for performance comparisons based on different use cases
and configurations. As a near term goal, there is the hope that this can help improve the performance of
[Cosmos SDK](https://github.com/cosmos/cosmos-sdk) based projects.

## `rhizome-db`

**WARNING: This is an experimental project. We make no guarantees it will
actually turn into anything useful.** Maybe it will, maybe it won't. Maybe some
other project will achieve the same goals in another way in the meantime, we
don't know.

A peer to peer databases which aims to eventually have the following features:
* can be run on any platform (server, mobile, web browser)
* syncs data between peers using CRDTs with eventually consistency
* has fine-grained read/write privacy and selective syncing strategies
* can also work with a light client (i.e. no local data just server connection)
* optional historical state and forking/branching like git, with pruning support
* merkle proofs for using the database at specific points in time as an immutable record
* SQL and GraphQL queries
* an RDF projection with SPARQL queries
* full transaction history with digital signatures
* authorization via various DID methods
* advanced CRDTs for collaborative editing (like Google Docs)
* geospatial and full text indexes
* should be as easy to use as any regular db people use like postgres, mongo, or sqlite
* good CPU, network, and storage performance
