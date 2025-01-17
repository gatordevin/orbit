# Packages

## Package Identifier

The package identifier (__pkgid__) is a project's unique string of characters following a specification. This specification is similiar to Xilinx's Vendor-Library-Name-Version (VLNV) format. When referencing a pkgid, each section is separated by a dot (`.`).

### Rules

The following rules apply to each section in the pkgid (vendor, library, and name):

- begins with an ASCII letter (`a-z`, `A-Z`)
- contains only ASCII letters (`a-z`, `A-Z`), ASCII digits (`0-9`), dashes `-`, and underscores `_`

### Fully Qualified Status

In some cases, a pkgid must be _fully qualified_, which means all 3 identifiers are provided: vendor, library, and name. For example, a pkgid must be fully qualified when creating a new one from the command-line.

### Specification Grammar

```
pkgid ::= [[<vendor>.]<library>.]<name>
```

If a pkgid can be determined by Orbit without specifying all parts, then unnecessary parts may be omitted. See examples for more information.

### Example Specifications
| pkgid                       | Vendor | Library | Name         
| -                           | -      | -       | -             
ks-tech.rary.gates            | ks-tech| rary    | gates       
uf.crypt.simon-cipher         | uf     | crypt   | simon-cipher
eel4712c.lab1                 |        | eel4712c| lab1        
lab2                          |        |         | lab2        
ks-tech..mips                 | ks-tech|         | mips   

### Namespace Collisions

Within a user's _catalog_, two different pkgids may share common identifiers, but cannot all be equivalent. Two identifiers are considered equivalent if their lowercase mapping is identical, where dashes `-` also map to underscores `_`.

- ks-tech.rary.gates __!=__ kstech.rary.gates
- ks-tech.rary.gates __==__ KS_TECH.RARY.GATES