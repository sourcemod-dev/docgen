import { Identifier } from './symbol';

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
