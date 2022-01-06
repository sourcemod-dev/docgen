import { IConstant } from './constant';
import { IDefine } from './define';
import { IEnumeration } from './enumeration';
import { IFunction } from './function';
import {
    IMethodMap
} from './method_map';
import { IProperty } from './property';
import { ITypeDefinition } from './type_def';
import { ITypeSet } from './type_set';
import {
    IEnumStruct,
    IField,
} from './enum_struct';

/* eslint-disable no-shadow */
export enum Identifier {
    Constant = 'constant',
    Define = 'define',
    Enumeration = 'enumeration',
    Function = 'function',
    MethodMap = 'methodmap',
    MethodMapMethod = 'methodmap_method',
    MethodMapProperty = 'methodmap_property',
    EnumStruct = 'enumstruct',
    EnumStructMethod = 'enumstruct_method',
    EnumStructField = 'enumstruct_field',
    Field = 'field',
    Property = 'property',
    TypeDefinition = 'typedef',
    TypeSet = 'typeset',
    Unknown = 'unknown',
}

// Weights smaller than 0.1
export enum IdentifierWeights {
    Enumeration = 0.03,
    Function = 0.02,
    MethodMap = 0.05,
    EnumStruct = 0.05,
    TypeDefinition = 0.03,
    TypeSet = 0.03,
}

export const SINGLETON_TYPES = [
    Identifier.Constant,
    Identifier.Define,
    Identifier.Function,
    Identifier.Enumeration,
    Identifier.MethodMap,
    Identifier.EnumStruct,
    Identifier.TypeDefinition,
    Identifier.TypeSet,
];

export const NESTED_TYPES = [
    Identifier.MethodMapMethod,
    Identifier.MethodMapProperty,
    Identifier.EnumStructMethod,
    Identifier.EnumStructField,
];

export type Symbol =
    IConstant |
    IDefine |
    IEnumeration |
    IFunction |
    IMethodMap |
    IProperty |
    IEnumStruct |
    IField |
    ITypeDefinition |
    ITypeSet;

export type SingletonSymbol = IConstant | IDefine | IFunction | IEnumeration | IMethodMap | IEnumStruct | ITypeDefinition | ITypeSet;

export type NestedSymbol = IProperty | IField;

export const PRIMITIVE_TYPES = [
    'int',
    'int&',
    'int[]',
    'bool',
    'bool&',
    'void',
    'const char[]',
    'char',
    'char[]',
    'const float',
    'float',
    'float&',
    'float[]',
];
