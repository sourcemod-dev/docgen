import { Function, MethodMap, EnumStruct, Constant, Define, Enumeration, TypeSet, TypeDefinition } from "./symbol";
import { Meta } from "./meta";
import { Source } from "./manifest";

export interface Bundle {
    /**
     * Meta descriptor of bundle content
     */
    meta: Meta;

    /**
     * Manifest source
     */
    source: Source;
}

export interface Strand {
    functions: Fibers<Function>,

    methodmaps: Fibers<MethodMap>,

    enumstructs: Fibers<EnumStruct>,

    constants: Fibers<Constant>,

    defines: Fibers<Define>,

    enums: Fibers<Enumeration>,

    typesets: Fibers<TypeSet>,

    typedefs: Fibers<TypeDefinition>,
}

export type Fibers<T> = Record<string, T>;

export interface Fiber<T> {
    symbol: T;

    /**
     * SVN version this symbol was introduced
     */
    created: Versioning | null;

    /**
     * SVN version this symbol was last modified
     */
    last_updated: Versioning | null;
}

export interface Versioning {
    hash: string;

    /**
     * Rev-list count
     * Mainly used for core where product.version will be within spec paths
     */
    count: number;

    /**
     * Unix timestamp, 64 bit, doesn't fit in JS number
     */
    time: string;
}
