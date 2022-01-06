import { IBundle, IFibers, IStrand, Meta, Source, IVersioning, Searchable } from '../interfaces';
import { Function, MethodMap, EnumStruct, Constant, Define, Enumeration, TypeDefinition, TypeSet } from './symbol';

export class Bundle {
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
     strands: Record<string, Strand>;
 
     /**
      * Current version this bundle was last parsed from
      * Chum bucket will continue from this commit
      */
     version: IVersioning | null;

    constructor(bundle: IBundle) {
        this.meta = bundle.meta;
        this.source = bundle.source;
        this.strands = Object.keys(bundle.strands).reduce((acc, strand) => {
            acc[strand] = new Strand(bundle.strands[strand]);
            return acc;
        }, {} as Record<string, Strand>);
        this.version = bundle.version;
    }
}

export class Strand {
    functions: IFibers<Function>;

    methodmaps: IFibers<MethodMap>;

    enumstructs: IFibers<EnumStruct>;

    constants: IFibers<Constant>;

    defines: IFibers<Define>;

    enums: IFibers<Enumeration>;

    typesets: IFibers<TypeSet>;

    typedefs: IFibers<TypeDefinition>;

    constructor(strand: IStrand) {
        this.functions = Strand.mapFibers(strand.functions, Function);
        this.methodmaps = Strand.mapFibers(strand.methodmaps, MethodMap);
        this.enumstructs = Strand.mapFibers(strand.enumstructs, EnumStruct);
        this.constants = Strand.mapFibers(strand.constants, Constant);
        this.defines = Strand.mapFibers(strand.defines, Define);
        this.enums = Strand.mapFibers(strand.enums, Enumeration);
        this.typesets = Strand.mapFibers(strand.typesets, TypeSet);
        this.typedefs = Strand.mapFibers(strand.typedefs, TypeDefinition);
    }

    private static mapFibers<T, F>(fibers: IFibers<T>, symbol: new (...args: any[]) => F): IFibers<F> {
        return Object.keys(fibers).reduce((acc, key) => {
            const fiber = fibers[key];
            acc[key] = {
                symbol: new symbol(fiber.symbol),
                created: fiber.created,
                last_updated: fiber.last_updated,
            };
            return acc;
        }, {} as IFibers<F>);
    }
}
