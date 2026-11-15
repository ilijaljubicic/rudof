# SHACL

## Constant RDF values

`sh:hasValue` and members of `sh:in` retain their RDF term identity when the
shapes graph is parsed and compiled. This includes IRIs, literals, blank nodes
and RDF 1.2 triple terms, including nested triple terms and directional literals.
For example:

```turtle
@prefix ex: <urn:example:> .
@prefix sh: <http://www.w3.org/ns/shacl#> .

ex:StatementShape a sh:PropertyShape ;
    sh:targetNode ex:claim ;
    sh:path ex:describes ;
    sh:hasValue <<( ex:subject ex:predicate ex:object )>> .
```

The required triple term is a value; its inner triple is not asserted by that
declaration. Blank-node constants compare by RDF identity, and directional
literals retain their direction. An enumeration conversion error fails parsing
instead of removing the affected member.

Validation supports these constants through native RDF objects. The data
generator currently reports an error for blank-node and triple-term constants
because its intermediate representation cannot preserve them.
