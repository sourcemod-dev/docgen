import { IBundle, IFibers, IStrand, Meta, Source, IVersioning, Searchable, SearchOptions, SearchResult } from '../interfaces';
import { Function, MethodMap, EnumStruct, Constant, Define, Enumeration, TypeDefinition, TypeSet } from './symbol';

export class Bundle implements IBundle, Searchable {
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

    public async search(needle: string, options: SearchOptions): Promise<SearchResult[]> {
        const ret: Promise<SearchResult[]>[] = [];

        for (const [include, strand] of Object.entries(this.strands)) {
            ret.push(strand.search(needle, {
                ...options,
                parents: [...options.parents, include],
            }));
        }

        return (await Promise.all(ret)).flat();
    }
}

export class Strand implements IStrand, Searchable {
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

    public async search(needle: string, options: SearchOptions): Promise<SearchResult[]> {
        const ret: Promise<SearchResult[]>[] = [];

        const searchSymbolType = (member: IFibers<Searchable>) => {
            for (const fiber of Object.values(member)) {
                ret.push(fiber.symbol.search(needle, options));
            }
        }

        searchSymbolType(this.functions);
        searchSymbolType(this.methodmaps);
        searchSymbolType(this.enumstructs);
        searchSymbolType(this.constants);
        searchSymbolType(this.defines);
        searchSymbolType(this.enums);
        searchSymbolType(this.typesets);
        searchSymbolType(this.typedefs);

        return (await Promise.all(ret)).flat();
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
