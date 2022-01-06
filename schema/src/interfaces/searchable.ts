import { Identifier } from './symbol';

export enum Part {
    Name = 'name',
    Parameter = 'parameter',
    Return = 'return',
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

    parents: string[];
}

export interface Searchable {
    search(needle: string, options: SearchOptions): Promise<SearchResult[]>;
}
