
import { GearApi, decodeAddress } from '@gear-js/api';
import { TypeRegistry } from '@polkadot/types';
import { TransactionBuilder, getServiceNamePrefix, getFnNamePrefix, ZERO_ADDRESS } from 'sails-js';

export type ActorId = string;

export interface Config {
  gas_to_delete_session: number | string | bigint;
  minimum_session_duration_ms: number | string | bigint;
  ms_per_block: number | string | bigint;
}

export interface IoHelloState {
  greeting: string;
  user_greetings: Array<[ActorId, string]>;
  counter: number | string | bigint;
}

export interface SignatureData {
  key: ActorId;
  duration: number | string | bigint;
  allowed_actions: Array<ActionsForSession>;
}

export type ActionsForSession = 'SayHello' | 'SayPersonalHello' | 'SetGreeting';

export interface SessionData {
  key: ActorId;
  expires: number | string | bigint;
  allowed_actions: Array<ActionsForSession>;
  expires_at_block: number;
}

export type Events =
  | { Hello: null }
  | { PersonalHello: { actor_id: ActorId; str: string } }
  | { GreetingSet: string };

const types = {
  Config: {
    gas_to_delete_session: 'u64',
    minimum_session_duration_ms: 'u64',
    ms_per_block: 'u64',
  },
  IoHelloState: {
    greeting: 'String',
    user_greetings: 'Vec<([u8;32], String)>',
    counter: 'u64',
  },
  SignatureData: {
    key: '[u8;32]',
    duration: 'u64',
    allowed_actions: 'Vec<ActionsForSession>',
  },
  ActionsForSession: {
    _enum: ['SayHello', 'SayPersonalHello', 'SetGreeting'],
  },
  SessionData: {
    key: '[u8;32]',
    expires: 'u64',
    allowed_actions: 'Vec<ActionsForSession>',
    expires_at_block: 'u32',
  },
  PersonalHello: {
    actor_id: '[u8;32]',
    str: 'String',
  },
  Events: {
    _enum: {
      Hello: 'Null',
      PersonalHello: 'PersonalHello',
      GreetingSet: 'String',
    },
  },
};

export class Program {
  public readonly registry: TypeRegistry;
  public readonly service: Service;
  public readonly session: Session;

  constructor(
    public api: GearApi,
    private _programId?: `0x${string}`,
  ) {
    this.registry = new TypeRegistry();
    this.registry.setKnownTypes({ types });
    this.registry.register(types);

    this.service = new Service(this);
    this.session = new Session(this);
  }

  public get programId(): `0x${string}` {
    if (!this._programId) throw new Error('Program ID is not set');
    return this._programId;
  }

  newCtorFromCode(code: Uint8Array | Buffer, config: Config): TransactionBuilder<null> {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'upload_program',
      ['New', config],
      '(String, Config)',
      'String',
      code,
    );
    this._programId = builder.programId;
    return builder;
  }

  newCtorFromCodeId(codeId: `0x${string}`, config: Config): TransactionBuilder<null> {
    const builder = new TransactionBuilder<null>(
      this.api,
      this.registry,
      'create_program',
      ['New', config],
      '(String, Config)',
      'String',
      codeId,
    );
    this._programId = builder.programId;
    return builder;
  }
}

export class Service {
  constructor(private _program: Program) {}

  public helloWorld(session_for_account: ActorId | null): TransactionBuilder<Events> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<Events>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'HelloWorld', session_for_account],
      '(String, String, Option<[u8;32]>)',
      'Events',
      this._program.programId,
    );
  }

  public personalHello(name: string, session_for_account: ActorId | null): TransactionBuilder<Events> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<Events>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'PersonalHello', name, session_for_account],
      '(String, String, String, Option<[u8;32]>)',
      'Events',
      this._program.programId,
    );
  }

  public setGreeting(new_greeting: string, session_for_account: ActorId | null): TransactionBuilder<Events> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<Events>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Service', 'SetGreeting', new_greeting, session_for_account],
      '(String, String, String, Option<[u8;32]>)',
      'Events',
      this._program.programId,
    );
  }

  public async queryCounter(originAddress?: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<bigint> {
    const payload = this._program.registry.createType('(String, String)', ['Service', 'QueryCounter']).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: originAddress ? decodeAddress(originAddress) : ZERO_ADDRESS,
      payload,
      value: value ?? 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    if (!reply.code.isSuccess) throw new Error(this._program.registry.createType('String', reply.payload).toString());
    const result = this._program.registry.createType('(String, String, u64)', reply.payload);
    return result[2].toBigInt();
  }

  public async queryGreeting(originAddress?: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<string> {
    const payload = this._program.registry.createType('(String, String)', ['Service', 'QueryGreeting']).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: originAddress ? decodeAddress(originAddress) : ZERO_ADDRESS,
      payload,
      value: value ?? 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    if (!reply.code.isSuccess) throw new Error(this._program.registry.createType('String', reply.payload).toString());
    const result = this._program.registry.createType('(String, String, String)', reply.payload);
    return result[2].toString();
  }

  public async queryState(originAddress?: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<IoHelloState> {
    const payload = this._program.registry.createType('(String, String)', ['Service', 'QueryState']).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: originAddress ? decodeAddress(originAddress) : ZERO_ADDRESS,
      payload,
      value: value ?? 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    if (!reply.code.isSuccess) throw new Error(this._program.registry.createType('String', reply.payload).toString());
    const result = this._program.registry.createType('(String, String, IoHelloState)', reply.payload);
    return result[2].toJSON() as IoHelloState;
  }

  public async queryUserGreeting(user: ActorId, originAddress?: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<string | null> {
    const payload = this._program.registry.createType('(String, String, [u8;32])', ['Service', 'QueryUserGreeting', user]).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: originAddress ? decodeAddress(originAddress) : ZERO_ADDRESS,
      payload,
      value: value ?? 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    if (!reply.code.isSuccess) throw new Error(this._program.registry.createType('String', reply.payload).toString());
    const result = this._program.registry.createType('(String, String, Option<String>)', reply.payload);
    return result[2].toJSON() as string | null;
  }

  public subscribeToHelloEvent(callback: (data: null) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) return;
      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Service' && getFnNamePrefix(payload) === 'Hello') {
        void Promise.resolve(callback(null)).catch(console.error);
      }
    });
  }

  public subscribeToPersonalHelloEvent(callback: (data: { actor_id: ActorId; str: string }) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) return;
      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Service' && getFnNamePrefix(payload) === 'PersonalHello') {
        void Promise.resolve(
          callback(
            this._program.registry.createType('(String, String, PersonalHello)', message.payload)[2].toJSON() as {
              actor_id: ActorId;
              str: string;
            }
          )
        ).catch(console.error);
      }
    });
  }

  public subscribeToGreetingSetEvent(callback: (data: string) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) return;
      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Service' && getFnNamePrefix(payload) === 'GreetingSet') {
        void Promise.resolve(
          callback(
            this._program.registry.createType('(String, String, String)', message.payload)[2].toString()
          )
        ).catch(console.error);
      }
    });
  }
}

export class Session {
  constructor(private _program: Program) {}

  public createSession(signature_data: SignatureData, signature: `0x${string}` | null): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Session', 'CreateSession', signature_data, signature],
      '(String, String, SignatureData, Option<Vec<u8>>)',
      'Null',
      this._program.programId,
    );
  }

  public deleteSessionFromAccount(): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Session', 'DeleteSessionFromAccount'],
      '(String, String)',
      'Null',
      this._program.programId,
    );
  }

  public deleteSessionFromProgram(session_for_account: ActorId): TransactionBuilder<null> {
    if (!this._program.programId) throw new Error('Program ID is not set');
    return new TransactionBuilder<null>(
      this._program.api,
      this._program.registry,
      'send_message',
      ['Session', 'DeleteSessionFromProgram', session_for_account],
      '(String, String, [u8;32])',
      'Null',
      this._program.programId,
    );
  }

  public async sessionForTheAccount(account: ActorId, originAddress?: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<SessionData | null> {
    const payload = this._program.registry.createType('(String, String, [u8;32])', ['Session', 'SessionForTheAccount', account]).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: originAddress ? decodeAddress(originAddress) : ZERO_ADDRESS,
      payload,
      value: value ?? 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    if (!reply.code.isSuccess) throw new Error(this._program.registry.createType('String', reply.payload).toString());
    const result = this._program.registry.createType('(String, String, Option<SessionData>)', reply.payload);
    return result[2].toJSON() as SessionData | null;
  }

  public async sessions(originAddress?: string, value?: number | string | bigint, atBlock?: `0x${string}`): Promise<Array<[ActorId, SessionData]>> {
    const payload = this._program.registry.createType('(String, String)', ['Session', 'Sessions']).toHex();
    const reply = await this._program.api.message.calculateReply({
      destination: this._program.programId,
      origin: originAddress ? decodeAddress(originAddress) : ZERO_ADDRESS,
      payload,
      value: value ?? 0,
      gasLimit: this._program.api.blockGasLimit.toBigInt(),
      at: atBlock,
    });
    if (!reply.code.isSuccess) throw new Error(this._program.registry.createType('String', reply.payload).toString());
    const result = this._program.registry.createType('(String, String, Vec<([u8;32], SessionData)>)', reply.payload);
    return result[2].toJSON() as Array<[ActorId, SessionData]>;
  }

  public subscribeToSessionCreatedEvent(callback: (data: null) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) return;
      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Session' && getFnNamePrefix(payload) === 'SessionCreated') {
        void Promise.resolve(callback(null)).catch(console.error);
      }
    });
  }

  public subscribeToSessionDeletedEvent(callback: (data: null) => void | Promise<void>): Promise<() => void> {
    return this._program.api.gearEvents.subscribeToGearEvent('UserMessageSent', ({ data: { message } }) => {
      if (!message.source.eq(this._program.programId) || !message.destination.eq(ZERO_ADDRESS)) return;
      const payload = message.payload.toHex();
      if (getServiceNamePrefix(payload) === 'Session' && getFnNamePrefix(payload) === 'SessionDeleted') {
        void Promise.resolve(callback(null)).catch(console.error);
      }
    });
  }
}
