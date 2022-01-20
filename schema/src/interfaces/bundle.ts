import { IFunction, IMethodMap, IEnumStruct, IConstant, IDefine, IEnumeration, ITypeSet, ITypeDefinition } from "./symbol";
import { Meta } from "./meta";
import { Source } from "./manifest";
import { IVersioning } from './metadata';

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
    functions: Record<string, IFunction>;

    methodmaps: Record<string, IMethodMap>;

    enumstructs: Record<string, IEnumStruct>;

    constants: Record<string, IConstant>;

    defines: Record<string, IDefine>;

    enums: Record<string, IEnumeration>;

    typesets: Record<string, ITypeSet>;

    typedefs: Record<string, ITypeDefinition>;
}
