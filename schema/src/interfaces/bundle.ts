import { IFunction, IMethodMap, IEnumStruct, IConstant, IDefine, IEnumeration, ITypeSet, ITypeDefinition } from "./symbol";
import { Meta } from "./meta";
import { Source } from "./manifest";

export interface IBundle {
    /**
     * Meta descriptor of bundle content
     */
    meta: Meta;

    /**
     * Manifest source
     */
    source: Source;

    /**
     * Strand or each individual include file
     * With optional addon metadata for versioning
     */
    strands: Record<string, IStrand>;

    /**
     * Current version this bundle was last parsed from
     * Chum bucket will continue from this commit
     */
    version: IVersioning | null;
}

export interface IStrand {
    functions: IFibers<IFunction>;

    methodmaps: IFibers<IMethodMap>;

    enumstructs: IFibers<IEnumStruct>;

    constants: IFibers<IConstant>;

    defines: IFibers<IDefine>;

    enums: IFibers<IEnumeration>;

    typesets: IFibers<ITypeSet>;

    typedefs: IFibers<ITypeDefinition>;
}

export type IFibers<T> = Record<string, IFiber<T>>;

export interface IFiber<T> {
    symbol: T;

    /**
     * SVN version this symbol was introduced
     */
    created: IVersioning | null;

    /**
     * SVN version this symbol was last modified
     */
    last_updated: IVersioning | null;
}

export interface IVersioning {
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
