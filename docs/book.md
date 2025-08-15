Introduction:


In the age of information where the information is created at scale we never seen before we will don’t struggle with problem of the access of information but their credibility and confidence.  [ please elaborate on that more, on the problem information trust, and reduction in the information age]
So in the day of AI any amount of information can be generated instantly which effectively  discredit the methods of judgments when “more = more likely correct”
The problem of trust and its distribution:

Lets imagine there is an animal.  The animal is seen by two people (observers) The one says “This animal is a dog” , second says: “This animal is a wolf”.
Whois is right?  Speaking from the perspective of knowledge theory (epistemology), we have, who make a judgment about the subject - the animal.

[ create here a diagram of two observers that look at the “animal”, One says: “This animal is a dog”, “This animal is a wolf”

The problem with these judgments is a certainty. We don’t know whose predication is correct and trusted. Both observations have a low level of belief and guarantee nothing but judgment
 The way to solve it would be through
- get more observers, and select the most popular judgment
- Find someone who is a specialist and can assert that this is a dog, not a wolf.
The first solution seems fair but doesn’t guarantee the majority could be wrong, because the differentiation of species might require special knowledge.
So the second solution claims that we could find a specialist. A specialist who can make a statement about the animal. Statement is predicate equipped with confidence and certainty about the subject (the animal).
So the person instead of predicating makes a statement that is associated with a high degree of belief.

Considering this, the second solution is more reliable; however, the problem remains: how can we identify entities that are competent to make statements instead of predications? One of the products that tries to address it is IOTA Hierarchies.
IOTA Hierarchies allows to create a Federation of “experts” that can make a predefined statements. Entities (accreditors) also have permissions to delegate the ability to make a credible statement to other trusted entities. Ultimately we end up with the concept of hierarchy of rights.

So IOTA Hierarchies isn’t about the attribution but mainly about distribution of attributes when we need complex dependencies and responsibilities delegations in large systems, where then necessary of delegation and revelation of delegation is required.

Now imagine the we have a Federation of Exeperts - Biologist, that specialize. The all create organization that collects all kinds of biology experts. But we have experts from different species not all of them are qualified to recognize every type of animal. Thanks to Federation and Hierchies we can distribute the the trust according to the area of the expertise of experts.

For instance:
	Federation of Biologists
[Federation of Biologists can recognise the animals of different species]
[ Create a language description and diagram in svg that creates a top-level (federation of biologist) that is divided into sub-unit that are able to rocgonize different types of animals. These sub-units are accreditors and can transfer (accredit) abilities to attest to the leaf-level entities like a scientist or biologist. Biologist only can recognise only specific types of animals]


Important:
So IOTA Hierarchies isn’s just about providing the atomic credential but mainly about way of distributing the

Problem of distribution exists in every aspect of our life. The whole civilisation relies on the delegation of duties and createion abstractions and oranisations that  comibined together can evolve and create things that small units cannot. So hierarchies and organistations are typical for homo-sapiens.

[ make it narrative]
The problem of distirbution of trust
companies
universities
government institutions
supply chains
military
medicine  etc
households and home-automations,  where AI assistants start to take over  the responsibilities for some aspects of our live



IOTA Hierachies main concepts:

The concept of Hierarchies. Hierarchies derives their name from the way that are distributed rights (accreditations) to accredit or attest. Everything starts from the Federation that is created by the entity.  Entity is an any object in IOTA network that can be distinguished by unique ID. So  Entity could be a Identity, could  be another package, orgranization, object, person, device.

In the Federation, entities who are part of federation can delegate (accredit) other entities to make statements (attest).  Federation also allows delegate the “ability to delegate" (ability to accredit ) to other entities. So we can observer the creation of naturally evolved hierarchies




Every Federation has so called Root authority. Root authority by default is the creator of the federation. Root autohity, except being the authority for all properties and values that are under the federation plays a role of “administrator” Every Federation must have at least one root authority. Root Authority is capable of managing the Federation. The Root authority can add or revoke the properties that are under the jurisdiction of federation.

The properties with their values are attributions that are used by accredited entities to make statements. Statement
is a combination of a property (predicate), a value(subject), and an object. For instance. Property is “animal.type”, Value: “cat”,  Target of attribution (object): living, specific animal.

[ create a mermaid class diagram that depiction the description above] -  composition of Statement, Attribution, Property  and its value.

Properties Shape:

The properties can have a shape. The shape is just a condition for the value that the property can take.
You can limit the property by following the constraints:

number
equal
greaterThank
lowerThan
String:
equal
startsWith
endsWith
contains
[Please create a table from constraints]





Accreditation:
	Accreditations, as the name suggests, it’s a process of delegating some abilities by one entity to another. In Federations, we can distinguish two types of accreditations:
Accreditation to attest
accreditation to accredit


Accreditation to attest is a process when one already accredited entity can assign the ability to make a statements that include the given set of attributes.

[ generate the mermaid diagram, where the one entity (actor) accredit_to_attest another actor. The actors becomes attester and can make a statements. (i.e This animal is a dog]


Accreditation to accredit - is a process when an already accredited entity can assign the ability to accredit other entities. So the receiver fo accreditation can accredit also accredit others to: attest or accredit


[ generate the mermaid diagram, where the one entity (actor) accredits another actor. The actor (receiver)  can accredit_to_attest or accredit_to_accredit the third actor]



[here it should be a entity relationship diagram in mermaid of typical hierarchy. Federation on the top, from federation the  ]



Identity.rs vs Hierarchies

Identitiy and Hierarchies at first look might be hard to distinguish.  Identity.rs  directly refers to the verifiable credentials. IOTA is an online implementation of Decentrialised Identity with verifiable credential. This concept focuses on presentation of the credentials in standarized way’’’

On the other hand IOTA hierarchies focus more on managing the distribution of unopinionetd properties which could be a credentials that are part of identity but it doesnt’t have to be. Hierarchies address the problem of delegation of rights defined in specification-agnostic manner,  Identity - on presentation of rights in unified manner.


So the first question could be can Identity.rs and Hierarchies could work together? Yes they can and there is no one way the can work due to of unopinionated nature of hierarchies


IOTA Identity.rs  as a root authority - to authorize the federation:
IOTA hierarchies doesn’t have any requirements regarding the entities that are parto of federation. So entities could be any move object that is distinguished by the ID. Identity.rs is no different. Assume you created an Identity that is that  this connected to your web2 domain through domain linkegage credentials.

So you by having Identity as the root authority automatically create a credibility and trust for entire federation

[create a entity relationship mermaid diagram ,that presents federation, then federation has root authority which is the Identity that is credible trough domain linkage. Then credible federation thanks to credible root authority can distribute attributions to other entities in form of statements]

IOTA Identity as any member of federation
following the assumption above about agosticity of entities that make the Federation, the Identity actually can be used by every member in the hierarchy. The every entity that is accredit must have an Identity , that increase the level  of credbility of hierarchy

[create a diagram in mermaid where you have a federation, and under the federation there is an root authorities, and intermediate accreditors, and leaf -nodes - attesters. All members are represented as IOTA Identity , instead of the normal object in IOTA network]


IOTA Hierarchies as a source of credentials in Identity.rs:
IMagine you have the IOTA Identity. IOTA Identity has credentials. These credentials are issued by another entity. So you can use a conventional resolver or you can use Hierachies to validate if given issuer is authorized to create credentials. In example we have a student with Verfiable Credentials, that contains the grades issued by the professor. Now while VALIDATING the verifiable presentation, the verifier can go to Federation that represents the university and check if processor was allowed to  issue the credential: university.grades.biology = 4

[ create a sequence diagram that describes the whole whole flow.  An student that has VC (verifialble credential). The VC contains the grade issued by proffesor.  Student presents to the employer. Employers make a request to the Federation and get response the professor was allowed to make statment, hence issue a credentials like university.grades.biology=4 ]




