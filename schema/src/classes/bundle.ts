import { IBundle, IStrand, Meta, Source, IVersioning, Searchable, SearchOptions, SearchResult, Identifier, Symbol, splitPath } from '../interfaces';
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

    public getSymbolByPath(path: string[]): Symbol {
        path = path.splice(0, 3);

        const strand: Strand = this.strands[path[0]];

        const L1 = splitPath(path[1]);

        let L1Symbol: Symbol;

        switch (L1.identifier) {
        case Identifier.Function:
            L1Symbol = strand.functions[L1.name];
            break;
        case Identifier.MethodMap:
            L1Symbol = strand.methodmaps[L1.name];
            break;
        case Identifier.EnumStruct:
            L1Symbol = strand.enumstructs[L1.name];
            break;
        case Identifier.Constant:
            L1Symbol = strand.constants[L1.name];
            break;
        case Identifier.Define:
            L1Symbol = strand.defines[L1.name];
            break;
        case Identifier.Enumeration:
            L1Symbol = strand.enums[L1.name];
            break;
        case Identifier.TypeSet:
            L1Symbol = strand.typesets[L1.name];
            break;
        default:
            L1Symbol = strand.typedefs[L1.name];
            break;
        }

        if (path.length === 2) {
            return L1Symbol;
        } else {
            if (![Identifier.MethodMap, Identifier.EnumStruct].includes(L1.identifier)) {
                return L1Symbol;
            }

            const L2 = splitPath(path[2]);

            const symbol = L1Symbol as MethodMap | EnumStruct;

            switch (L2.identifier) {
            case Identifier.MethodMapMethod:
            case Identifier.Function:
                return symbol.methods[L2.name];
            case Identifier.EnumStructField:
            case Identifier.Field:
                return (symbol as EnumStruct).fields[L2.name];
            // case Identifier.MethodMapProperty:
            // case Identifier.Property:
            default:
                return (symbol as MethodMap).properties[L2.name];
            }
        }
    }
}

export class Strand implements IStrand, Searchable {
    functions: Record<string, Function>;

    methodmaps: Record<string, MethodMap>;

    enumstructs: Record<string, EnumStruct>;

    constants: Record<string, Constant>;

    defines: Record<string, Define>;

    enums: Record<string, Enumeration>;

    typesets: Record<string, TypeSet>;

    typedefs: Record<string, TypeDefinition>;

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

        const searchSymbolType = (member: Record<string, Searchable>) => {
            for (const f of Object.values(member)) {
                ret.push(f.search(needle, options));
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

        // Return at least somewhat similar results
        return (await Promise.all(ret)).flat().filter(e => e.score > 0.5);
    }

    private static mapFibers<T, F>(fibers: Record<string, T>, symbol: new (...args: any[]) => F): Record<string, F> {
        return Object.keys(fibers).reduce((acc, key) => {
            acc[key] = new symbol(fibers[key]);
            return acc;
        }, {} as Record<string, F>);
    }
}
