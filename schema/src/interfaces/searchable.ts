import { Identifier, normalizeIdentifier } from './symbol';

export enum Part {
    Name,
    Parameter,
    Return,
}

export interface SearchResult {
    name: string;

    identifier: Identifier;

    part: Part;

    path: string[];

    score: number;
}

export interface SearchOptions {
    weighted?: boolean;

    identifier?: Identifier;

    l1Only?: boolean;

    parents: string[];
}

export interface Searchable {
    search(needle: string, options: Readonly<SearchOptions>): Promise<SearchResult[]>;
}

export function normalizePath(path: string[]): string[] {
    return path.map(e => {
        if (!e.includes('.')) {
            return e;
        }

        const parts = e.split('.');

        return `${normalizeIdentifier(parts[0] as Identifier)}.${parts[1]}`;
    });
}

export function getPathIdentifier(path: string): Identifier {
    const parts = path.split('.');

    return parts[0] as Identifier;
}

export function identifierToStrandProp(identifier: Identifier): string {
    switch (identifier) {
        case Identifier.Constant:
            return 'constants';
        case Identifier.Define:
            return 'defines';
        case Identifier.Function:
            return 'functions';
        case Identifier.Enumeration:
            return 'enums';
        case Identifier.MethodMap:
            return 'methodmaps';
        case Identifier.EnumStruct:
            return 'enumstructs';
        case Identifier.TypeDefinition:
            return 'typedefs';
        case Identifier.TypeSet:
            return 'typesets';
        default:
            throw new Error(`Unknown identifier: ${identifier}`);
    }
}
