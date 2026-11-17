# AVM2 Verification and Optimization

In order to run AVM2 bytecode most efficiently, we first *verify* it, to ensure
certain properties about the bytecode (e.g. that all locals accessed are within
bounds), and then we *optimize* it, removing unnecessary coercions and early
binding property accesses whenever possible.

The primary purpose of the verifier and optimizer is to improve AVM2
performance. Their secondary purpose is to improve correctness. The
[avmplus optimizer](https://github.com/adobe/avmplus/blob/master/core/Verifier.cpp)
is written in such a way that many of its optimizations are observable.
Unfortunately, some SWF files depend on this behavior.

There are several optimization passes, each of which contribute some
optimizations to the code. These are run in sequence in `core/src/avm2/optimizer.rs`.

<b>Note!</b>

The type-aware optimizer pass performs some type verification of its own. Due to
avmplus's design, this type verification is difficult to separate from the
optimizer. From here on, this will be referred to as <b>type verification</b>,
separate from the initial verification pass, which will simply be referred to as
<b>verification</b>.

The difference between the two is that <b>type verification</b> requires keeping
track of the current state of the code at all times (i.e. abstract
interpretation), while <b>verification</b> only requires being aware of the control
flow of the code (and only at a minimal level).

## Table of Contents

* [avmplus](#avmplus)
* [Verification](#verification)
* [Peephole 1](#peephole-1)
* [Type-Aware Optimization](#type-aware-optimization)
    * [Early Binding](#early-binding)
    * [Coercion Removal](#coercion-removal)
    * [Scope Optimizations](#scope-optimizations)
    * [Algorithm](#algorithm)
* [Peephole 2](#peephole-2)
* [Dead Store Elimination](#dead-store-elimination)
* [Peephole 3](#peephole-3)
* [Cleanup](#cleanup)

## avmplus

It might be helpful to reference the verifier and optimizer that avmplus uses.
See https://github.com/adobe/avmplus/blob/master/core/Verifier.cpp

Despite being titled "Verifier", avmplus's `Verifier` class performs
verification, type verification, and optimization all at once. Notably, it does
not perform any peephole optimizations or dead store elimination; instead, it
relies on its JIT for those.

## Verification

This stage is equivalent to the first part of avmplus's [verification and optimization pass](https://github.com/adobe/avmplus/blob/master/core/Verifier.cpp).

AVM2 bytecode is first read through the verifier, in `verify.rs`. The verifier
translates the method's code to a linear sequence of AVM2 ops, in our custom
representation. This custom representation varies from ordinary AVM2 bytecode in
a few ways:

 - some ops from ordinary AVM2 bytecode are split into two equivalent ops (see examples below)
 - some ops from ordinary AVM2 bytecode are merged (see examples below)
 - some new ops are added (e.g. `StoreLocal`, which acts like `SetLocal` but does not pop from the stack)
 - all constant pool entries are resolved

All subsequent operations operate on these ops.

Here are some of the important differences that the the custom representation has from ordinary AVM2 bytecode:
 - Branch ops, such as `IfLt`, are always split into a comparison and branch (e.g. `LessThan` and `IfTrue`)
 - `GetLex` is split into `FindPropStrict` and `GetProperty`
 - `CallSuperVoid` is split into `CallSuper` and `Pop`
 - `PushByte` and `PushShort` are replaced by an equivalent `PushInt`
 - `GetGlobalScope` is replaced by either `GetOuterScope` or `GetScopeObject`
 - `GetGlobalSlot` is split into two ops: first, either `GetOuterScope` or `GetScopeObject`; and second, `GetSlot`
 - `GetProperty` is replaced by one of three specialized implementations: `GetPropertyStatic`, `GetPropertyFast`, or `GetPropertySlow` (depending on the `Multiname`)
 - The same is done for `SetProperty`

The verifier does not translate unreachable code, or even consider it at all.
Consider the following AVM2 bytecode, which has unreachable code, a sequence of
`PushByte`s:

```
max_stack: 2
max_locals: 1

    GetLocal 0
    PushScope
    Jump label
    PushByte 1
    PushByte 2
    PushByte 3
    PushByte 4
    PushByte 5
    PushByte 6
label:
    FindPropStrict "trace"
    GetLocal 0
    CallPropVoid "trace"
    ReturnVoid
```

When translating code, the verifier follows the control flow of the code. This
means that it will never view unreachable code. This is observable! The
unreachable code in this example would overflow the stack declared by the
method. However, the verifier does not consider the code at all, so this method
passes verification. Some obfuscated SWFs will even include random bytes in
unreachable paths, so it is important that the verifier skips such code.

While translating code, the verifier will perform some quick verification
checks:
 - the op is actually a valid op
 - the constant pool entries that the op references are valid
 - local registers that the op references are valid
 - `NewActivation` ops are only present in methods that are marked `NEED_ACTIVATION`
 - `Dxns` and `DxnsLate` ops are only present in methods that are marked `NEED_DXNS`
 - etc

The verifier then resolves jump offsets, resulting in bytecode that the
interpreter is capable of running.

However, type verification has not been run on the bytecode yet, which means
that running this bytecode could crash the interpreter. In order to run type
verification, we need to enter the optimizer.

<b>Note!</b>

From this point on, the bytecode is a linear sequence of ops, with all branches
referencing indices of ops. This sequence of ops is not in any particular order
with regards to the control flow graph; it simply represents the order of the
ops as they appeared in the bytecode.

## Peephole 1

The first "optimization" pass, once verification ends, is a simple pass that
removes no-op jumps. Any `Jump`, `IfTrue`, or `IfFalse` ops with an offset of
zero are removed. This significantly simplifies the control flow graph of
functions in some obfuscated SWFs.

## Type-Aware Optimization

This is the main optimization pass. It also performs type verification.

This pass involves abstract interpretation over the bytecode. The abstract
values used to represent real values are fairly simple: they store only the
AVM2 class of each value, as well as some extra information which is useful for
certain specific optimizations (e.g. "is the value guaranteed to be not-null?").

If an abstract value at a point has a class `T`, it is guaranteed that the real
value (when the code is interpreted) can be successfully coerced to class `T`
without altering the value. Coercion refers to the `Value::coerce_to_type`
method, which is used in, for example, the implementation of the `Coerce` op.

Of course, it is not guaranteed that the value will actually be of class `T`
when the bytecode is run in the interpreter. Consider the following example:

```
public class Fruit { }

public class Apple extends Fruit { }

public class Test {
    public function Test() {
        var myApple:Apple = new Apple();

        processFruit(myApple);
    }

    public function processFruit(fruit:Fruit):void {
        // ...
    }
}
```

At the beginning of `processFruit`, the abstract interpreter considers local #1
(the local register that the `fruit` argument has been loaded into) to be of
type `Fruit`. As in this example, however, the `fruit` argument can be of any
subtype of `Fruit`, including `Apple`.

Abstract values are determined through a variety of methods:
 - Argument types upon method entry
 - Semantics of AVM2 ops
 - Declared return types and field types in other classes

The optimizer will visit all blocks to determine the true possible types at each
point in the code. At the same time, it will perform type verification, to ensure
the bytecode is valid. For example, it will ensure that:
 - the stack never overflows or underflows
 - `GetSlot` and `SetSlot` ops are always valid
 - `GetScopeObject` ops are always within bounds

Now that type verification is complete and the optimizer is aware of all types,
the type-aware optimizer can perform various optimizations on the code.

### Early binding

One of the most important optimizations the optimizer performs is early binding.
Early binding allows for property accesses using `GetProperty`, `SetProperty`,
`InitProperty`, and `CallProperty` to be resolved before the method is executed,
reducing the runtime overhead of such property accesses.

For example, consider the following code:

```
public class Point {
    public var x:Number; // slot #0
    public var y:Number; // slot #1
}

public class Line {
    public var p1:Point; // slot #0
    public var p2:Point; // slot #1

    public function lengthSquared():Number {
        var xDiff:Number = this.p1.x - this.p2.x;
        var yDiff:Number = this.p1.y - this.p2.y;
        return xDiff * xDiff + yDiff * yDiff;
    }
}
```

The `lengthSquared` method may be compiled down into the following bytecode:

```
max_stack: 2
max_locals: 1

    GetLocal 0
    PushScope

    GetLocal 0
    GetProperty "p1"
    GetProperty "x"
    GetLocal 0
    GetProperty "p2"
    GetProperty "x"
    Subtract
    SetLocal 1

    GetLocal 0
    GetProperty "p1"
    GetProperty "y"
    GetLocal 0
    GetProperty "p2"
    GetProperty "y"
    Subtract
    SetLocal 2

    GetLocal 1
    GetLocal 1
    Multiply
    GetLocal 2
    GetLocal 2
    Multiply
    Add
    ReturnValue
```

Each `GetProperty` op in this code is slow to run because it requires a vtable
property resolution. However, thanks to how AS3 encourages using type
information, the field that each `GetProperty` op is going to resolve to can
easily be determined statically.

Therefore, the optimizer can transform this code to the following:

```
max_stack: 2
max_locals: 1

    GetLocal 0
    PushScope

    GetLocal 0
    GetSlot 0
    GetSlot 0
    GetLocal 0
    GetSlot 1
    GetSlot 0
    Subtract
    SetLocal 1

    GetLocal 0
    GetSlot 0
    GetSlot 1
    GetLocal 0
    GetSlot 1
    GetSlot 1
    Subtract
    SetLocal 2

    GetLocal 1
    GetLocal 1
    Multiply
    GetLocal 2
    GetLocal 2
    Multiply
    Add
    ReturnValue
```

Each `GetProperty` op has been optimized to a `GetSlot`, which directly accesses
the `slot`. `GetSlot` runs about five times as fast as `GetProperty`, so this
code is significantly faster now.

If the property were a getter, the `GetProperty` op could also be optimized to
`CallMethod`, which executes a method based on dispatch id rather than
performing a full property lookup.

A similar optimization can be applied to `SetProperty` and `InitProperty`: it
can be optimized to `SetSlot` or `CallMethod` when the property it is referring
to can be statically determined.

### Coercion Removal

Besides early binding, the optimizer can also perform removal of redundant
coercions.

For example, consider the earlier mentioned optimization from `SetProperty` and
`InitProperty` to `SetSlot`. While `SetSlot` is significantly faster than
`SetProperty` and `InitProperty`, it is still slow because it requires coercing
the value about to be stored into the slot to the correct type.

When possible, the optimizer optimizes `SetProperty` and `InitProperty` even
further to the custom op `SetSlotNoCoerce`. This only happens when the optimizer
is *also* able to determine that no coercion is needed before storing the value
in the slot. In well-typed code, this optimization activates for the majority of
static `SetProperty` and `InitProperty` ops.

Besides `SetProperty`, the optimizer also considers removing all of the built-in
AVM2 coercion ops if possible. These include `CoerceB`, `CoerceD`, `CoerceI`,
`CoerceS`, `CoerceU`, and `Coerce`.

For example, consider the following code:

```
var banana:Banana = new Banana();
```

This code may be compiled down into the following bytecode:

```
findpropstrict "Banana"
constructprop "Banana"
coerce "Banana"
```

The optimizer can remove the `coerce` op from the bytecode, as it is guaranteed
that constructing the `Banana` class yields a value of type `Banana`.

### Scope Optimizations

The optimizer can also statically resolve the scope ops `FindPropStrict` and
`FindProperty` when possible, removing the overhead of performing this
resolution at runtime.

`FindPropStrict` and `FindPropStrict` can be optimized to:
 - `GetScopeObject` if the target property was found in the local scope stack
 - `GetOuterScope` if the target property was found in the outer scope chain
 - `GetScriptGlobals` if the target property was found in domain scripts

These ops are all significantly faster than `FindPropStrict` and `FindProperty`.

### Algorithm

(<b>note</b>: this section is pretty technical! I recommend skipping straight to
[the second peephole optimization pass](#peephole-2))

The optimizer begins by splitting the bytecode slice into blocks. Unlike a basic
block, which has one entry-point and one exit-point (its end), these blocks have
one entry-point and multiple exit-points.

The first step is to perform type analysis and type verification. This is done
using an algorithm that follows the control flow of the code.

The optimizer declares two structures: a list of abstract states at the
entry-point of each block, and a worklist of blocks that need to be processed
(or re-processed).

Initially, the abstract states in the list are blank; the first time the block
is visited by the abstract interpreter, the abstract state will be filled out.
The abstract interpreter will broaden the abstract state if it is visited again
(past the first time) with different types. If it is visited with the same
types, or if the new types it is visited with do not broaden the abstract state
at all, the recorded abstract state remains unchanged.

The optimizer starts at the entry block, and begins abstract interpretation.
Every time it encounters a branch, it fills out/broadens the known abstract
state at the entry-point of the block the branch targets. If the abstract state
at the entry-point of the block has changed (either been filled out or
broadened), it will add the block to the worklist of blocks to visit, since it
is now necessary to perform type verification on the block's code again.

This algorithm is run in a loop that repeatedly pops from the worklist and
processes blocks, running type verification and gathering type information.

Once all blocks have been visited with their broadest possible abstract states
(which implies the worklist is now empty), optimization begins. This is done
using an algorithm that linearly processes blocks.

Since the types of all values are now known exactly (as much as possible), this
step processes the list of blocks linearly. The optimizer begins with the entry
block, setting the current abstract state to the state recorded for the entry
block. It then performs abstract interpretation on the block, but ignores all
branches and jumps. Once the block's end has been reached, the optimizer then
repeats this step for the second block. It completely replaces the current
abstract state with the state recorded for the second block, and again performs
abstract interpretation on the second block's code. This process is repeated for
each block.

During these final abstract interpetation passes, optimizations are actually
performed. Once each block has been processed by this final change, the type-
aware optimizer pass is complete.

## Peephole 2

The type-aware optimizer performs some of the most effective optimizations, but
it has one major downside: it can only operate on single ops. This means that it
cannot perform any optimizations on pairs of ops, such as the elimination of

```
    GetLocal 0
    Pop
```

To eliminate such sequences, a peephole pass is run after the type-aware
optimizer pass.

This pass introduces the `StoreLocal` op when it would be faster to use it.
`StoreLocal` stores a value into a local register without popping it from the
stack.

The pass performs five major optimizations:
 - Elimination of `PushScope` when the method never reads from the scope stack
 - Elimination of pure push + pop pairs, such as the aforementioned `GetLocal` + `Pop`
 - Elimination of `CoerceB` before `IfTrue`, `IfFalse`, and `Not`, as it's redundant
 - Replacement of `SetLocal X` + `GetLocal X` and `Dup` + `SetLocal X` with `StoreLocal X`
 - Replacement of `Add` and `Subtract` with `AddI` and `SubtractI` when possible

## Dead Store Elimination

After the second peephole pass, a dead store elimination pass is done. This pass
eliminates `Kill`, `SetLocal`, and `StoreLocal` ops when the local is never
read from after the store. It specifically tracks local register writes.

Ops that write to local registers are important for the dead store elimination
pass, for obvious reasons. These ops include `Kill`, `SetLocal`, `StoreLocal`,
`DecLocal`, `DecLocalI`, `IncLocal`, `IncLocalI`, and `HasNext2`. Such ops will
be referred to as "<b>writing ops</b>" from now on. Similarly, ops that read
from local registers- `GetLocal`, `DecLocal`, `DecLocalI`, `IncLocal`,
`IncLocalI`, and `HasNext2`- are also important for the dead store elimination
pass, and will be referred to as "<b>reading ops</b>" from now on.

This pass runs in three separate stages.

First, the indices of the writing ops that wrote to each local register at each
point are tracked.

Consider the following code:

```
0   PushByte 1
1   SetLocal 1

2   PushByte 1
3   PushByte 3
4   Add
5   SetLocal 2

6   Kill 3

7   PushByte 5
8   SetLocal 3

9   FindPropStrict "trace"
10  GetLocal 1
11  CallPropVoid "trace"
```

After op #1, local register #1 has been written to by op #1. After op #5, local
register #2 has been written to by op #5. After op #6, local register #3 has
been written to by op #6. After op #8, local register #3 has been written to by
op #8 (op #6's result is no longer relevant, as op #8 is the most recent write).

In branching code, multiple writing ops can be tracked as writing to one local
register. Consider the following code:

```
if (a < 0) {
    b = 3;
} else {
    b = 4;
}
trace(b);
```

By the time the `trace` statement is called, both the `b = 3` store and the
`b = 4` store are tracked as writing to the local register that `b` references.

This step uses the same algorithm that the type-aware optimizer uses to merge
states, except it merges local store information instead of entire abstract
states.

After this step is complete, and local store information at the entry-point of
each block has been gathered, the dead store elimination pass iterates linearly
over all ops, just like the type-aware optimizer. Using the local store
information previously gathered, a map of each writing op and whether its store
is known to be live (i.e. if its store was later read from) is created. Each
writing op stores its index in the map and initially reports its store as dead.
Once a reading op reads from that specific instance of a write, its store is
considered live.

Once all blocks have been iterated over, and the map is complete, all writing
ops in the map that have a dead store are eliminated. `Kill` and `StoreLocal`
are replaced by `Nop`, and `SetLocal` is replaced with `Pop`. The increment and
decrement writing ops, along with `HasNext2`, are never eliminated. They can
have side effects such as throwing errors due to failed coercions, so they
cannot be removed.

## Peephole 3

After the dead store elimination pass, the peephole pass is run again, to clean
up the dead code such as the following lying around:

```
    PushByte 0
    Pop        // Was previously SetLocal 1
    PushByte 0
    Pop        // Was previously SetLocal 2
    PushDouble NaN
    Pop        // Was previously SetLocal 3
    PushByte 0
    Pop        // Was previously SetLocal 4
```

## Cleanup

At this point, the main optimization passes are complete, and only some cleanup
remains to be done.

Every single one of the previous optimizations operates on a slice of ops, not a
`Vec` of ops. This means that ops can never be completely removed from the code,
only replaced with `Nop` ops. After all the optimization passes, it is not
uncommon for over a third of the ops in the code to be `Nop`.

Executing all these `Nop` ops in the interpreter does have non-negligible
overhead, so at this point in the optimizer, they are removed from the code.
To avoid this, the final optimizer pass considers the entire `Vec` of ops and
actually removes all `Nop` ops in it. This requires rewriting branch offsets,
since the indices of code move around.

Once this pass is complete, there are no `Nop` ops remaining in the code.
Optimization is now complete.
